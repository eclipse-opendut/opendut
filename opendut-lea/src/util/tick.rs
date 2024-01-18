use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Tick(u128);

impl Tick {

    pub fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl From<u128> for Tick {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl From<Tick> for u128 {
    fn from(value: Tick) -> Self {
        value.0
    }
}

impl PartialEq<u128> for Tick {
    fn eq(&self, other: &u128) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<u128> for Tick {
    fn partial_cmp(&self, other: &u128) -> Option<Ordering> {
        Some(self.0.cmp(other))
    }
}

impl Display for Tick {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
