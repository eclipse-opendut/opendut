use std::{env, io::{Cursor, ErrorKind, Write}, path::PathBuf, process::Stdio};

use opendut_types::peer::executor::{ContainerCommand, ContainerCommandArgument, ContainerDevice, ContainerEnvironmentVariable, ContainerImage, ContainerName, ContainerPortSpec, ContainerVolume, Engine, ResultsUrl};

use tokio::{fs::{self, File}, io::{AsyncBufReadExt, AsyncReadExt, BufReader}, process::{Child, Command}, sync::{mpsc, watch}};
use tracing::{error, warn, info};
use url::Url;
use uuid::Uuid;
use walkdir::WalkDir;
use zip::{write::{FileOptionExtension, FileOptions, SimpleFileOptions}, CompressionMethod, ZipWriter};
use anyhow::Result;

use crate::service::test_execution::webdav_client::{self, WebdavClient};


#[derive(Debug)]
enum ContainerState {
    Created,
    Running,
    Restarting,
    Exited,
    Paused,
    Dead,
}

pub struct ContainerConfiguration {
    pub name: ContainerName,
    pub engine: Engine,
    pub image: ContainerImage,
    pub command: ContainerCommand,
    pub args: Vec<ContainerCommandArgument>,
    pub envs: Vec<ContainerEnvironmentVariable>,
    pub ports: Vec<ContainerPortSpec>,
    pub devices: Vec<ContainerDevice>,
    pub volumes: Vec<ContainerVolume>,
    pub results_url: Option<ResultsUrl>,
}

pub struct ContainerManager{
    config: ContainerConfiguration,
    container_name: Option<String>,
    results_dir: PathBuf,
    webdav_client: WebdavClient,
    log_reader: Option<ContainerLogReader>,
    termination_channel_rx: watch::Receiver<bool>,
}

const MONITOR_INTERVAL_MS: u64 = 1000;
const RESULTS_READY_FILE: &str = ".results_ready";
const CONTAINER_RESULTS_DIRECTORY: &str = "/results";

impl ContainerManager {

    pub fn new(container_configuration: ContainerConfiguration, termination_channel_rx: watch::Receiver<bool>) -> Self {
        Self { 
            config: container_configuration,
            container_name: None,
            results_dir: env::temp_dir().join(format!("opendut-edgar-results_{}", Uuid::new_v4())),
            webdav_client: WebdavClient::new("some_dummy_token".to_string()), // TODO: Authenticate with actual token
            log_reader: None,
            termination_channel_rx
        }
    }

    pub async fn start(&mut self) {
        match self.run().await {
            Ok(_) => (),
            Err(cause) => error!("{}", cause.to_string()),
        }
    }

    async fn run(&mut self) -> Result<(), Error> {
        let mut results_uploaded = false;

        self.create_results_dir().await?;
        self.start_container().await?;
        self.log_reader = Some(
            ContainerLogReader::create(
                self.config.engine.to_string(), 
                self.container_name.as_ref().unwrap().clone()
            )?);

        loop {
            self.log_reader.as_mut().unwrap().read().await;

            // If the value in the channel has changed or the channel has been closed, we terminate
            if self.termination_channel_rx.has_changed().unwrap_or(true) {
                self.stop_container().await?;
            }

            if self.are_results_ready().await? {
                self.remove_result_ready_indicator().await?;
                self.upload_results().await?;
                results_uploaded = true;
            }

            match self.get_container_state().await? {
                ContainerState::Running => (),
                ContainerState::Exited => {
                    if ! results_uploaded {
                        self.remove_result_ready_indicator().await?;
                        self.upload_results().await?;
                    }
                    break
                },
                state => {
                    warn!("Unexpected container state of '{}': {:?}", self.config.name, state)
                },
            }

            tokio::time::sleep(std::time::Duration::from_millis(MONITOR_INTERVAL_MS)).await;
        }

        self.cleanup_results_dir().await?;

        Ok(())
    }

    async fn get_container_state(&self) -> Result<ContainerState, Error> {
        match &self.container_name {
            Some(container_name) => {
                let output = Command::new(&self.config.engine.to_string())
                    .args(["inspect", "-f", "'{{.State.Status}}'", container_name])
                    .output()
                    .await
                    .map_err(|cause| Error::CommandLineProgramExecution { command: format!("{} inspect", &self.config.engine.to_string()), cause })?;
                
                match String::from_utf8_lossy(&output.stdout).into_owned().replace('\'', "").trim() {
                    "created" => Ok(ContainerState::Created),
                    "running" => Ok(ContainerState::Running),
                    "restarting" => Ok(ContainerState::Restarting),
                    "exited" => Ok(ContainerState::Exited),
                    "paused" => Ok(ContainerState::Paused),
                    "dead" => Ok(ContainerState::Dead),
                    unknown_state => Err(Error::Other { message: format!("Unknown container state returned by {} inspect: '{}'", &self.config.engine.to_string(), unknown_state) } ),
                }
            },
            None => Err(Error::Other { message: "get_container_state() called without container_name present".to_string()}),
        }
        
    }

    async fn start_container(&mut self) -> Result<(), Error>{

        let mut cmd = Command::new(self.config.engine.to_string());
        cmd.arg("run");
        cmd.arg("--detach");
        cmd.arg("--net=host");

        // TODO: Determining the name like this and then creating the container is theoretically susceptible to race conditions
        let mut container_name = String::new();
        for ctr in 1..i32::MAX {
            container_name = format!("{}-{}", self.config.name, ctr);
            if ! self.check_container_name_exists(&container_name).await? {
                break;
            }
        }
        cmd.args(["--name", container_name.as_str()]);
        self.container_name = Some(container_name);

        cmd.args(["--mount", format!("type=bind,source={},target={}", self.results_dir.to_string_lossy(), CONTAINER_RESULTS_DIRECTORY).as_str()]);
        
        for env in &self.config.envs {
            cmd.args(["--env", &format!("{}={}", env.name(), env.value())]);
        }
        for port in &self.config.ports {
            cmd.args(["--publish", port.value()]);
        }
        for volume in &self.config.volumes {
            cmd.args(["--volume", volume.value()]);
        }
        for device in &self.config.devices {
            cmd.args(["--devices", device.value()]);
        }

        cmd.arg(&self.config.image.to_string());

        if let ContainerCommand::Value(command) = &self.config.command {
            cmd.arg(command.as_str());
        }

        for arg in &self.config.args {
            cmd.arg(arg.to_string());
        }
        let output = cmd.output()
            .await
            .map_err(|cause| Error::CommandLineProgramExecution { command: format!("{} run", &self.config.engine.to_string()), cause })?;

        match output.status.success() {
            true => {
                info!("Started container {}", self.config.name);
                Ok(())
            },
            false => Err(Error::Other { message: format!("Starting container '{}' failed: {}", self.config.name, String::from_utf8_lossy(&output.stderr)).to_string()})
        }

    }

    async fn check_container_name_exists(&self, name: &str) -> Result<bool, Error>{
        let output = Command::new(&self.config.engine.to_string())
            .args(["container", "inspect", name])
            .output()
            .await
            .map_err(|cause| Error::CommandLineProgramExecution { command: format!("{} inspect", &self.config.engine.to_string()), cause })?;

        Ok(output.status.success())
    }

    async fn stop_container(&self) -> Result<(), Error>{
        match &self.container_name {
            Some(container_name) => {
                let output = Command::new(&self.config.engine.to_string())
                    .args(["stop", container_name])
                    .output()
                    .await
                    .map_err(|cause| Error::CommandLineProgramExecution { command: format!("{} stop", &self.config.engine.to_string()), cause })?;

                match output.status.success() {
                    true => Ok(()),
                    false => Err(Error::Other { message: format!("Stopping container failed: {}", String::from_utf8_lossy(&output.stderr)).to_string()})
                }

            },
            None => Err(Error::Other { message: "stop_container() called without container_name present".to_string()}),
        }
    }

    async fn remove_result_ready_indicator(&self) -> Result<(), Error>{ 
        let mut indicator_file = self.results_dir.clone();
        indicator_file.push(RESULTS_READY_FILE);
        match fs::remove_file(&indicator_file).await {
            Ok(_) => Ok(()),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => Ok(()),
                _ => Err(Error::Other { message: format!("Failed to remove result indicator file '{}': {}", indicator_file.to_string_lossy(), err) }),
            },
        }

    }

    async fn upload_results(&self) -> Result<(), Error>{
        info!("Starting upload for results of {}", self.config.name);
        let results_url = match &self.config.results_url {
            Some(results_url) => results_url.value(),
            None => {
                info!("Container {} has no results URL, won't upload results.", self.config.name);
                return Ok(());
            },
        };
        
        let mut zipped_data = Vec::new();
        let zip_options = SimpleFileOptions::default().compression_method(CompressionMethod::BZIP2);
        create_zip_from_directory(&mut zipped_data, &self.results_dir, zip_options).await.map_err(|cause| Error::ResultZipping { path: self.results_dir.clone(), cause })?;

        self.webdav_client.create_collection_path(results_url.clone())
            .await
            .map_err(|cause| Error::ResultUploadingInternal { url: results_url.clone(), cause })?;

        let results_file_url = results_url.join(
            format!("{}_{}.zip", chrono::offset::Local::now().format("%Y-%m-%d_%H-%M-%S"), self.config.name).as_str()
        ).map_err(|cause| Error::Other { message: format!("Failed to construct URL for results directory: {}", cause) })?;

        let response = self.webdav_client.put(zipped_data, results_file_url.clone())
            .await
            .map_err(|cause| Error::ResultUploadingInternal { url: results_file_url.clone(), cause })?;

        match response.status().is_success() {
            true => {
                info!("Successfully uploaded results of {}", self.config.name);
                Ok(())
            },
            false => Err(Error::ResultUploadingServer { container_name: self.config.name.clone(), url: results_file_url.clone(), status: response.status() }),
        }

    }

    async fn create_results_dir(&mut self) -> Result<(), Error>{
        fs::create_dir(&self.results_dir)
            .await
            .map_err(|cause| Error::Other { message: format!("Failed to create results directory '{}': {}", self.results_dir.to_string_lossy(), cause) })?;
        Ok(())
    }

    async fn cleanup_results_dir(&self) -> Result<(), Error> {
        fs::remove_dir_all(&self.results_dir)
            .await
            .map_err(|cause| Error::Other { message: format!("Failed to remove results directory '{}': {}", self.results_dir.to_string_lossy(), cause) })?;
        Ok(())
    }

    async fn are_results_ready(&self) -> Result<bool, Error> {
        let mut indicator_file = self.results_dir.clone();
        indicator_file.push(RESULTS_READY_FILE);
        Ok(indicator_file.is_file())
    }

}

async fn create_zip_from_directory<T>(data: &mut Vec<u8>, directory: &PathBuf, file_options: FileOptions<'_, T>) -> Result<()> 
    where
        T: FileOptionExtension + std::marker::Copy,
    {
        let mut file_buffer = Vec::new();
        let zip_buffer = Cursor::new(data);
        let mut zip = ZipWriter::new(zip_buffer);

        for entry_res in WalkDir::new(directory) {
            let entry = entry_res?;
            let entry_path = entry.path();
            let entry_metadata = entry.metadata()?;

            if entry_metadata.is_file() {
                let mut f = File::open(&entry_path).await?;
                f.read_to_end(&mut file_buffer).await?;
                let relative_path = entry_path.strip_prefix(directory)?;
                zip.start_file(relative_path.to_string_lossy(), file_options)?;
                zip.write_all(file_buffer.as_ref())?;
                file_buffer.clear();
            } else if entry_metadata.is_dir() {
                let relative_path = entry_path.strip_prefix(directory)?;
                zip.add_directory(relative_path.to_string_lossy(), file_options)?;
            }
        }

        Ok(())
    }

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failure while invoking command line program '{command}': {cause}")]
    CommandLineProgramExecution { command: String, cause: std::io::Error },
    #[error("Failure while creating a ZIP archive of the test results at '{path}' : {cause}")]
    ResultZipping { path: PathBuf, cause: anyhow::Error },
    #[error("Failure while uploading test results to '{url}': {cause}")]
    ResultUploadingInternal { url: Url, cause: webdav_client::Error },
    #[error("Failure while uploading test results for '{container_name}' to '{url}' (HTTP status {status})")]
    ResultUploadingServer { container_name: ContainerName, url: Url, status: reqwest::StatusCode },
    #[error("{message}")]
    Other { message: String },
}

struct ContainerLogReader {
    _log_proc: Child,
    receiver: mpsc::Receiver<Vec<u8>>,
}

impl ContainerLogReader {
    pub fn create(engine: String, container_name: String) -> Result<Self, Error> {
        let mut cmd = Command::new(&engine);
        cmd.args(["logs", "--timestamps", "--follow"]);
        cmd.arg(container_name);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);

        let mut child = cmd.spawn()
            .map_err(|cause| Error::CommandLineProgramExecution { command: format!("{engine} logs"), cause })?;

        let stdout = child.stdout.take().ok_or(Error::Other { message: format!("Failed to get stdout of '{engine} logs' process")})?;

        let mut stdout_reader = BufReader::new(stdout);

        let (tx, rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);

        tokio::spawn(async move {
            let mut buffer = Vec::new();
    
            loop {
                match stdout_reader.read_until(b'\n', &mut buffer).await {
                    Ok(0) => {
                        // EOF reached
                        break;
                    }
                    Ok(_) => {
                        let _ = tx.send(buffer.clone()).await;
                        buffer.clear();
                    }
                    Err(e) => {
                        error!("Error reading from logs stdout stream: {}", e);
                        break;
                    }
                }
            }
        });
        

        Ok(
            Self {
                _log_proc: child,
                receiver: rx,
            }
        )
    }


    async fn read(&mut self) {
        while let Ok(line) = self.receiver.try_recv() {
            info!("Received line: {:?}", String::from_utf8_lossy(&line))
        }
    }
    
}