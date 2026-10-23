use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use chrono::Local;
use clap::{ArgAction, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::core::docker::command::DockerCommand;
use crate::core::project::ProjectRootDir;

const DEFAULT_TRIVY_VOLUME: &str = "trivy-cache";
const DEFAULT_TRIVY_IMAGE: &str = "aquasec/trivy:latest";

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum AuditOutputFormat {
    #[default]
    Table,
    Markdown,
    Json,
}

#[derive(Debug, clap::Args)]
pub struct AuditCli {
    /// Output format for the extracted image list
    #[arg(long, short, value_enum, default_value_t = AuditOutputFormat::Table)]
    pub format: AuditOutputFormat,

    /// Run Trivy vulnerability scan on all extracted images
    #[arg(long, action = ArgAction::SetTrue)]
    pub scan: bool,

    /// Skip updating the cached Trivy vulnerability DB before scanning
    #[arg(long, action = ArgAction::SetTrue)]
    pub skip_db_update: bool,

    /// Filter Trivy scan results by severity (e.g. HIGH,CRITICAL)
    #[arg(long)]
    pub severity: Option<String>,

    /// Output directory for Trivy scan reports
    #[arg(long)]
    pub output_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct ComposeConfig {
    #[serde(default)]
    services: BTreeMap<String, ComposeService>,
}

#[derive(Debug, Deserialize)]
struct ComposeService {
    image: Option<String>,
    build: Option<ComposeBuild>,
}

#[derive(Debug, Deserialize)]
struct ComposeBuild {
    context: PathBuf,
    dockerfile: PathBuf,
    #[serde(default)]
    args: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ServiceImageInfo {
    pub service: String,
    pub image: String,
    pub source: ImageSource,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ImageSource {
    ComposeImage,
    DockerfileFrom { dockerfile: PathBuf },
    DockerfileCopyFrom { dockerfile: PathBuf },
}

impl AuditCli {
    pub fn run(self) -> anyhow::Result<()> {
        let images = extract_localenv_images()?;

        match self.format {
            AuditOutputFormat::Table => print_images_table(&images),
            AuditOutputFormat::Markdown => print_images_markdown(&images),
            AuditOutputFormat::Json => {
                let json = serde_json::to_string_pretty(&images)?;
                println!("{json}");
            }
        }

        if self.scan {
            let out_dir = self.output_dir.unwrap_or_else(|| {
                PathBuf::project_path_buf().join("target/ci/trivy")
            });
            run_trivy_scans(&images, &out_dir, self.skip_db_update, self.severity.as_deref(), self.format)?;
        }

        Ok(())
    }
}

pub fn extract_localenv_images() -> anyhow::Result<Vec<ServiceImageInfo>> {
    let mut cmd = DockerCommand::new();
    cmd.add_localenv_args();
    if std::env::var("OPENDUT_DOCKER_IMAGE_HOST").ok().as_deref() == Some("docker-registry.example.com") {
        cmd.env("OPENDUT_DOCKER_IMAGE_HOST", "ghcr.io");
    }
    if std::env::var("OPENDUT_DOCKER_IMAGE_NAMESPACE").ok().as_deref() == Some("docker-opendut-namespace") {
        cmd.env("OPENDUT_DOCKER_IMAGE_NAMESPACE", "eclipse-opendut");
    }
    let output = cmd
        .arg("config")
        .arg("--format")
        .arg("json")
        .expect_output("Failed to resolve localenv docker-compose configuration.")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("docker compose config failed: {stderr}"));
    }

    let compose_config: ComposeConfig = serde_json::from_slice(&output.stdout)
        .context("Failed to deserialize `docker compose config --format json` output.")?;

    let project_root = PathBuf::project_path_buf();
    let mut results = Vec::new();

    for (service_name, service) in compose_config.services {
        if let Some(build) = service.build {
            let dockerfile_abs = if build.dockerfile.is_absolute() {
                build.dockerfile.clone()
            } else {
                build.context.join(&build.dockerfile)
            };
            let dockerfile_display = dockerfile_abs
                .strip_prefix(&project_root)
                .unwrap_or(&dockerfile_abs)
                .to_path_buf();

            let content = fs::read_to_string(&dockerfile_abs)
                .with_context(|| format!("Failed to read Dockerfile at {:?}", dockerfile_abs))?;

            for (image, is_copy_from) in parse_dockerfile_images(&content, &build.args) {
                let source = if is_copy_from {
                    ImageSource::DockerfileCopyFrom { dockerfile: dockerfile_display.clone() }
                } else {
                    ImageSource::DockerfileFrom { dockerfile: dockerfile_display.clone() }
                };
                results.push(ServiceImageInfo {
                    service: service_name.clone(),
                    image,
                    source,
                });
            }
        } else if let Some(image) = service.image {
            results.push(ServiceImageInfo {
                service: service_name,
                image,
                source: ImageSource::ComposeImage,
            });
        }
    }

    Ok(results)
}

fn parse_dockerfile_images(
    dockerfile_content: &str,
    build_args: &BTreeMap<String, String>,
) -> Vec<(String, bool)> {
    let mut resolved_args = BTreeMap::new();
    let mut stage_aliases = HashSet::new();
    let mut extracted = Vec::new();

    for raw_line in dockerfile_content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        match tokens[0].to_ascii_uppercase().as_str() {
            "ARG" if tokens.len() >= 2 => {
                let arg_def = tokens[1..].join(" ");
                if let Some((key, default_val)) = arg_def.split_once('=') {
                    resolved_args.insert(
                        key.trim().to_string(),
                        default_val.trim().trim_matches('"').to_string(),
                    );
                }
            }
            "FROM" => {
                for (k, v) in build_args {
                    resolved_args.insert(k.clone(), v.clone());
                }
                let mut idx = 1;
                while idx < tokens.len() && tokens[idx].starts_with("--") {
                    idx += 1;
                }
                if idx < tokens.len() {
                    let raw_image = tokens[idx];
                    let resolved_image = substitute_vars(raw_image, &resolved_args);

                    if idx + 2 < tokens.len() && tokens[idx + 1].eq_ignore_ascii_case("AS") {
                        stage_aliases.insert(tokens[idx + 2].to_string());
                    }

                    if !resolved_image.eq_ignore_ascii_case("scratch")
                        && !stage_aliases.contains(&resolved_image)
                    {
                        extracted.push((resolved_image, false));
                    }
                }
            }
            "COPY" => {
                for (k, v) in build_args {
                    resolved_args.insert(k.clone(), v.clone());
                }
                for token in &tokens[1..] {
                    if let Some(from_val) = token.strip_prefix("--from=") {
                        let resolved = substitute_vars(from_val, &resolved_args);
                        if !stage_aliases.contains(&resolved)
                            && !resolved.eq_ignore_ascii_case("scratch")
                        {
                            extracted.push((resolved, true));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let mut deduped = Vec::new();
    for item in extracted {
        if !deduped.contains(&item) {
            deduped.push(item);
        }
    }
    deduped
}

fn substitute_vars(input: &str, vars: &BTreeMap<String, String>) -> String {
    let mut result = input.to_string();
    for (key, val) in vars {
        result = result.replace(&format!("${{{key}}}"), val);
        result = result.replace(&format!("${key}"), val);
    }
    result
}

fn format_image_source(source: &ImageSource) -> String {
    match source {
        ImageSource::ComposeImage => "docker-compose.yml (image)".to_string(),
        ImageSource::DockerfileFrom { dockerfile } => {
            format!("{} (FROM)", dockerfile.display())
        }
        ImageSource::DockerfileCopyFrom { dockerfile } => {
            format!("{} (COPY --from)", dockerfile.display())
        }
    }
}

fn print_images_table(images: &[ServiceImageInfo]) {
    println!("{:<22} {:<56} SOURCE", "SERVICE", "IMAGE");
    println!("{}", "-".repeat(105));
    for item in images {
        println!(
            "{:<22} {:<56} {}",
            item.service,
            item.image,
            format_image_source(&item.source)
        );
    }
}

fn print_images_markdown(images: &[ServiceImageInfo]) {
    println!("| Service | Image | Source |");
    println!("|---|---|---|");
    for item in images {
        println!(
            "| `{}` | `{}` | `{}` |",
            item.service,
            item.image,
            format_image_source(&item.source)
        );
    }
}

fn sanitize_filename(image: &str) -> String {
    image
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[derive(Debug, Deserialize)]
struct TrivyReport {
    #[serde(rename = "Results", default)]
    results: Vec<TrivyResult>,
}

#[derive(Debug, Deserialize)]
struct TrivyResult {
    #[serde(rename = "Vulnerabilities", default)]
    vulnerabilities: Option<Vec<TrivyVulnerability>>,
}

#[derive(Debug, Deserialize)]
struct TrivyVulnerability {
    #[serde(rename = "VulnerabilityID")]
    vulnerability_id: String,
    #[serde(rename = "Severity")]
    severity: String,
    #[serde(rename = "FixedVersion")]
    fixed_version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ImageScanSummary {
    image: String,
    report_file: String,
    critical: usize,
    high: usize,
    medium: usize,
    low: usize,
    fixable_critical: usize,
    fixable_high: usize,
    top_cves: Vec<String>,
}

fn summarize_trivy_report(image: &str, report_file: &str, report_path: &Path) -> anyhow::Result<ImageScanSummary> {
    let content = fs::read_to_string(report_path)
        .with_context(|| format!("Failed to read Trivy report at {:?}", report_path))?;
    let report: TrivyReport = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse Trivy JSON report at {:?}", report_path))?;

    let mut critical = 0;
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;
    let mut fixable_critical = 0;
    let mut fixable_high = 0;
    let mut top_cves = Vec::new();
    let mut seen_cves = HashSet::new();

    for result in report.results {
        for vuln in result.vulnerabilities.unwrap_or_default() {
            let has_fix = vuln
                .fixed_version
                .as_deref()
                .is_some_and(|f| !f.trim().is_empty());

            match vuln.severity.as_str() {
                "CRITICAL" => {
                    critical += 1;
                    if has_fix {
                        fixable_critical += 1;
                    }
                    if top_cves.len() < 5 && seen_cves.insert(vuln.vulnerability_id.clone()) {
                        top_cves.push(vuln.vulnerability_id);
                    }
                }
                "HIGH" => {
                    high += 1;
                    if has_fix {
                        fixable_high += 1;
                    }
                    if top_cves.len() < 5 && seen_cves.insert(vuln.vulnerability_id.clone()) {
                        top_cves.push(vuln.vulnerability_id);
                    }
                }
                "MEDIUM" => medium += 1,
                "LOW" => low += 1,
                _ => {}
            }
        }
    }

    Ok(ImageScanSummary {
        image: image.to_string(),
        report_file: report_file.to_string(),
        critical,
        high,
        medium,
        low,
        fixable_critical,
        fixable_high,
        top_cves,
    })
}

fn format_scan_summary_markdown(timestamp: &str, summaries: &[ImageScanSummary]) -> String {
    let mut out = String::new();
    out.push_str(&format!("### Trivy Scan Summary ({timestamp})\n\n"));
    out.push_str("| Image | CRITICAL (Fixable) | HIGH (Fixable) | MEDIUM | LOW | Top CRITICAL/HIGH CVEs |\n");
    out.push_str("|---|---:|---:|---:|---:|---|\n");
    for s in summaries {
        let cves = if s.top_cves.is_empty() {
            "-".to_string()
        } else {
            s.top_cves.join(", ")
        };
        out.push_str(&format!(
            "| `{}` | {} ({}) | {} ({}) | {} | {} | {} |\n",
            s.image, s.critical, s.fixable_critical, s.high, s.fixable_high, s.medium, s.low, cves
        ));
    }
    out
}

fn print_scan_summary_table(summaries: &[ImageScanSummary]) {
    println!(
        "\n{:<56} {:>10} {:>10} {:>8} {:>8}  TOP CVEs",
        "IMAGE", "CRIT(FIX)", "HIGH(FIX)", "MEDIUM", "LOW"
    );
    println!("{}", "-".repeat(120));
    for s in summaries {
        let crit = format!("{} ({})", s.critical, s.fixable_critical);
        let high = format!("{} ({})", s.high, s.fixable_high);
        let cves = if s.top_cves.is_empty() {
            "-".to_string()
        } else {
            s.top_cves.join(", ")
        };
        println!(
            "{:<56} {:>10} {:>10} {:>8} {:>8}  {}",
            s.image, crit, high, s.medium, s.low, cves
        );
    }
}

fn run_trivy_scans(
    images: &[ServiceImageInfo],
    output_dir: &Path,
    skip_db_update: bool,
    severity: Option<&str>,
    format: AuditOutputFormat,
) -> anyhow::Result<()> {
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory {:?}", output_dir))?;

    DockerCommand::new()
        .arg("volume")
        .arg("create")
        .arg(DEFAULT_TRIVY_VOLUME)
        .expect_status("Failed to ensure Trivy cache Docker volume", false)?;

    if !skip_db_update {
        println!("Updating Trivy vulnerability database in volume '{DEFAULT_TRIVY_VOLUME}'...");
        DockerCommand::new()
            .arg("run")
            .arg("--rm")
            .arg("-v")
            .arg(format!("{DEFAULT_TRIVY_VOLUME}:/root/.cache"))
            .arg(DEFAULT_TRIVY_IMAGE)
            .arg("image")
            .arg("--download-db-only")
            .expect_show_status("Failed to download Trivy vulnerability DB")?;

        DockerCommand::new()
            .arg("run")
            .arg("--rm")
            .arg("-v")
            .arg(format!("{DEFAULT_TRIVY_VOLUME}:/root/.cache"))
            .arg(DEFAULT_TRIVY_IMAGE)
            .arg("image")
            .arg("--download-java-db-only")
            .expect_show_status("Failed to download Trivy Java vulnerability DB")?;
    }

    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let unique_images: BTreeSet<&str> = images.iter().map(|i| i.image.as_str()).collect();
    let out_dir_abs = fs::canonicalize(output_dir)?;
    let mut summaries = Vec::new();

    for image in unique_images {
        let filename = format!("{}_{}.json", timestamp, sanitize_filename(image));
        let report_path = output_dir.join(&filename);
        println!("Scanning {image} -> {}", report_path.display());

        let mut cmd = DockerCommand::new();
        cmd.arg("run")
            .arg("--rm")
            .arg("-v")
            .arg(format!("{DEFAULT_TRIVY_VOLUME}:/root/.cache"))
            .arg("-v")
            .arg("/var/run/docker.sock:/var/run/docker.sock")
            .arg("-v")
            .arg(format!("{}:/out", out_dir_abs.display()))
            .arg(DEFAULT_TRIVY_IMAGE)
            .arg("image")
            .arg("--skip-db-update")
            .arg("--skip-java-db-update");

        if let Some(sev) = severity {
            cmd.arg("--severity").arg(sev);
        }

        cmd.arg("--format")
            .arg("json")
            .arg("--output")
            .arg(format!("/out/{filename}"))
            .arg(image)
            .expect_show_status(&format!("Trivy scan failed for image '{image}'"))?;

        if let Ok(summary) = summarize_trivy_report(image, &filename, &report_path) {
            summaries.push(summary);
        }
    }

    let summary_json_path = output_dir.join(format!("{timestamp}_audit-summary.json"));
    fs::write(&summary_json_path, serde_json::to_string_pretty(&summaries)?)?;

    let markdown_summary = format_scan_summary_markdown(&timestamp, &summaries);
    let summary_md_path = output_dir.join(format!("{timestamp}_audit-summary.md"));
    fs::write(&summary_md_path, &markdown_summary)?;

    match format {
        AuditOutputFormat::Table => {
            print_scan_summary_table(&summaries);
            println!("\nMarkdown table:\n\n{markdown_summary}");
        }
        AuditOutputFormat::Markdown => {
            println!("\n{markdown_summary}");
        }
        AuditOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&summaries)?);
        }
    }

    println!("Trivy reports written to: {}", output_dir.display());
    println!("  JSON summary:     {}", summary_json_path.display());
    println!("  Markdown summary: {}", summary_md_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multistage_dockerfile_and_substitutes_args() {
        let dockerfile = r#"
ARG KEYCLOAK_VERSION
FROM quay.io/keycloak/keycloak:$KEYCLOAK_VERSION AS builder
FROM registry.access.redhat.com/ubi9 AS ubi-micro-build
FROM quay.io/keycloak/keycloak:$KEYCLOAK_VERSION
COPY --from=builder /opt/keycloak/ /opt/keycloak/
COPY --from=ubi-micro-build /mnt/rootfs /
COPY --from=ghcr.io/tarampampam/curl:8.16.0 /bin/curl /bin/curl
"#;
        let mut args = BTreeMap::new();
        args.insert("KEYCLOAK_VERSION".to_string(), "26.7.5".to_string());

        let images = parse_dockerfile_images(dockerfile, &args);
        assert_eq!(
            images,
            vec![
                ("quay.io/keycloak/keycloak:26.7.5".to_string(), false),
                ("registry.access.redhat.com/ubi9".to_string(), false),
                ("ghcr.io/tarampampam/curl:8.16.0".to_string(), true),
            ]
        );
    }
}
