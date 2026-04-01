use opendut_util::project;
use tracing::info;
use crate::interactive_message;

#[derive(Clone, PartialEq, Eq)]
pub enum DryRun { Yes, No }
impl DryRun {
    pub fn not(&self) -> bool {
        self == &DryRun::No
    }

    fn force_dry_run_in_development(&mut self) {
        if project::is_running_in_development() {
            interactive_message!("{DEVELOPMENT_DRY_RUN_BANNER}");
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
