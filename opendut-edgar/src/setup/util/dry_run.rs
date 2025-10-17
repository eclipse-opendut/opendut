use opendut_util::project;
use tracing::info;

#[derive(Clone, PartialEq, Eq)]
pub enum DryRun { Yes, No }
impl DryRun {
    pub fn not(&self) -> bool {
        self == &DryRun::No
    }

    fn force_dry_run_in_development(&mut self) {
        if project::is_running_in_development() {
            println!("{DEVELOPMENT_DRY_RUN_BANNER}");
            info!("{DEVELOPMENT_DRY_RUN_BANNER}");
            *self = DryRun::Yes;
        }
    }
}
impl std::str::FromStr for DryRun {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let dry_run = bool::from_str(value)?;

        let mut dry_run = if dry_run { DryRun::Yes } else { DryRun::No };
        dry_run.force_dry_run_in_development();

        if dry_run.not() {
            sudo::with_env(&["OPENDUT_EDGAR_"]) //Request before doing anything else, as it restarts the process when sudo is not present.
                .expect("Failed to request sudo privileges.");
        }
        Ok(dry_run)
    }
}

const DEVELOPMENT_DRY_RUN_BANNER: &str = r"
                Running in
             Development mode
                   ----
          Activating --dry-run to
        prevent changes to the system.
        ";
