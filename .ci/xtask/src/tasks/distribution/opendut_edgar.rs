use crate::{Arch, Package};


#[tracing::instrument]
pub fn collect_edgar_specific_files(package: &Package, target: &Arch) -> anyhow::Result<()> {

    netbird::get_client_artifact(package, target)?;

    licenses::get(package, target)?;

    Ok(())
}


mod netbird {
    use std::fs;
    use std::path::PathBuf;

    use anyhow::anyhow;

    use crate::{Arch, Package};

    #[tracing::instrument]
    pub fn get_client_artifact(package: &Package, target: &Arch) -> anyhow::Result<()> {
        //Modelled after documentation here: https://docs.netbird.io/how-to/getting-started#binary-install

        let metadata = crate::metadata::cargo();
        let version = metadata.workspace_metadata["ci"]["netbird"]["version"].as_str().clone()
            .ok_or(anyhow!("NetBird version not defined."))?;

        let os = "linux";

        let arch = match target {
            Arch::X86 => "amd64",
            Arch::Arm64 => "arm64",
            Arch::Armhf => "armv6",
        };

        let folder_name = format!("v{version}");
        let file_name = format!("netbird_{version}_{os}_{arch}.tar.gz");

        let netbird_artifact = download_dir().join(&folder_name).join(&file_name);
        fs::create_dir_all(&netbird_artifact.parent().unwrap())?;

        if !netbird_artifact.exists() { //download
            let url = format!("https://github.com/reimarstier/netbird/releases/download/{folder_name}/{file_name}");

            println!("Downloading netbird_{version}_{os}_{arch}.tar.gz...");
            let bytes = reqwest::blocking::get(url)?
                .error_for_status()?
                .bytes()?;
            println!("Retrieved {} bytes.", bytes.len());

            fs::write(&netbird_artifact, bytes)
                .map_err(|cause| anyhow!("Error while writing to '{}': {cause}", netbird_artifact.display()))?;
        }
        assert!(netbird_artifact.exists());

        let out_file = distribution_file(package, target);
        fs::create_dir_all(&out_file.parent().unwrap())?;

        fs::copy(&netbird_artifact, &out_file)
            .map_err(|cause| anyhow!("Error while copying from '{}' to '{}': {cause}", netbird_artifact.display(), out_file.display()))?;

        Ok(())
    }

    fn download_dir() -> PathBuf {
        crate::constants::target_dir().join("netbird")
    }

    pub fn distribution_file(package: &Package, target: &Arch) -> PathBuf {
        crate::tasks::distribution::distribution_dir().join(target.triple()).join(package.ident()).join("install").join("netbird.tar.gz")
    }
}

mod licenses {
    use std::fs;
    use std::path::PathBuf;

    use crate::{Arch, Package};

    #[tracing::instrument]
    pub fn get(package: &Package, target: &Arch) -> anyhow::Result<()> {

        let licenses_file = crate::tasks::licenses::generate_licenses(package)?;

        let out_dir = out_dir(package, target);
        let licenses_file_name = format!("{}.licenses.json", package.ident());
        fs::create_dir_all(&out_dir)?;

        fs::copy(
            &licenses_file,
            &out_dir.join(&licenses_file_name)
        )?;

        Ok(())
    }

    fn out_dir(package: &Package, target: &Arch) -> PathBuf {
        crate::tasks::distribution::distribution_dir().join(target.triple()).join(package.ident()).join("licenses")
    }
}
