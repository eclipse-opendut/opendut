use crate::common::TestSuiteIdentifier;
use std::fmt::{Display, Formatter};

pub struct SourceCode {
    pub identifier: TestSuiteIdentifier,
    pub code: String,
    pub version: ApiVersion,
}

#[derive(Clone, Copy, Debug)]
pub enum ApiVersion {
    V1_0,
}

impl Display for ApiVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiVersion::V1_0 => write!(f, "1.0"),
        }
    }
}
