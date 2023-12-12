#!/usr/bin/env rust-script
//! Dependencies can be specified in the script file itself as follows:
//!
//! ```cargo
//! [dependencies]
//! reqwest = { version = "0.11.22", features = ["blocking", "json"] }
//! ```

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::blocking;
use std::collections::HashMap;

fn main() {
    let keycloak_url = "http://192.168.32.204";
    let mgmt_api="192.168.32.211/api/users";
    let resp = reqwest::blocking::get("{}/realms/master/protocol/openid-connect/token")
        .json::<HashMap<String, String>>();
}
