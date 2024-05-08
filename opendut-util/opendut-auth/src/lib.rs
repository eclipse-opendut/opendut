use cfg_if::cfg_if;

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
