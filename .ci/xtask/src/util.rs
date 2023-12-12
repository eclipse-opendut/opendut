use std::fmt::{Display, Formatter};
use std::process::Command;
use std::str::FromStr;

use anyhow::Result;


pub fn install_crate(name: impl AsRef<str>) -> Result<()> {
    Command::new("cargo")
        .arg("install")
        .arg(name.as_ref())
        .status()?;
    Ok(())
}


#[derive(Clone, Debug)]
pub struct Package(String);
impl Package {
    pub fn name(&self) -> String {
        self.0.to_owned()
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for Package {
    type Err = String;
    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Package(value.into()))
    }
}
