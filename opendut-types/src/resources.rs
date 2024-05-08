use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::fmt;
use crate::peer::PeerId;

#[derive(Copy, Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Id {
    value: Uuid,
}

impl Id {
    pub fn random() -> Self {
        Uuid::new_v4().into()
    }
    pub fn value(&self) -> Uuid {
        self.value
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

impl From<PeerId> for Id {
    fn from(value: PeerId) -> Self {
        Self {
            value: value.0,
        }
    }
}

impl From<Uuid> for Id {
    fn from(value: Uuid) -> Self {
        Self {
            value,
        }
    }
}
