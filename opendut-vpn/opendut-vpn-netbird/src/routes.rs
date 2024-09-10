use reqwest::Url;

use crate::netbird;

pub fn setup_keys(base_url: Url) -> Url {
    join(base_url, "setup-keys")
}

pub fn groups(base_url: Url) -> Url {
    join(base_url, "groups")
}

pub fn group(base_url: Url, group_id: &netbird::GroupId) -> Url {
    join(groups(base_url), &group_id.0)
}

pub fn peers(base_url: Url) -> Url {
    join(base_url, "peers")
}

pub fn peer(base_url: Url, peer_id: &netbird::PeerId) -> Url {
    join(peers(base_url), &peer_id.0)
}

pub fn policies(base_url: Url) -> Url {
    join(base_url, "policies")
}

pub fn policy(base_url: Url, policy_id: &netbird::PolicyId) -> Url {
    join(policies(base_url), &policy_id.0)
}

fn join(mut base_url: Url, path: &str) -> Url {
    base_url.path_segments_mut()
        .map(|mut path_segments| {
            path_segments
                .pop_if_empty()
                .push(path);
        })
        .unwrap_or_else(|_| panic!("Base URL '{}' is not valid. It must be a fully qualified URL, like 'https://example.com:1234/a/b'.", base_url.clone()));
    base_url
}


#[cfg(test)]
mod tests {
    use googletest::prelude::*;

    use super::*;

    #[test]
    fn should_join_without_trailing_slash() -> anyhow::Result<()> {

        let url = Url::parse("https://localhost:1234/api")?;

        let result = join(url, "other");
        assert_that!(result.as_str(), eq("https://localhost:1234/api/other"));

        Ok(())
    }

    #[test]
    fn should_join_with_trailing_slash() -> anyhow::Result<()> {

        let url = Url::parse("https://localhost:1234/api/")?;

        let result = join(url, "other");
        assert_that!(result.as_str(), eq("https://localhost:1234/api/other"));

        Ok(())
    }
}
