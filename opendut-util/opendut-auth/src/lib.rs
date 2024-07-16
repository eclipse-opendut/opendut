use cfg_if::cfg_if;
use chrono::TimeDelta;

const TOKEN_GRACE_PERIOD: TimeDelta = TimeDelta::seconds(10);

cfg_if! {
    if #[cfg(feature = "public_client")] {
        pub mod public;
    }
}

cfg_if! {
    if #[cfg(any(feature = "confidential_client", feature = "registration_client"))] {
        pub mod confidential;
    }
}
cfg_if! {
    if #[cfg(feature = "registration_client")] {
        pub mod registration;
    }
}
