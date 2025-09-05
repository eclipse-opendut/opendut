use opendut_types::cluster::{IllegalClusterId, ClusterId};
use opendut_types::peer::{IllegalPeerId, PeerId};

pub trait UrlEncodable {
    fn url_encode(&self) -> String;
}

#[allow(unused)]
pub trait UrlDecodable<T, E> {
    fn url_decode(encoded: &str) -> Result<T, E>;
}

impl UrlEncodable for ClusterId {
    fn url_encode(&self) -> String {
        self.to_string()
    }
}

impl UrlDecodable<ClusterId, IllegalClusterId> for ClusterId {
    fn url_decode(encoded: &str) -> Result<ClusterId, IllegalClusterId> {
        ClusterId::try_from(encoded)
    }
}

impl UrlEncodable for PeerId {
    fn url_encode(&self) -> String {
        self.to_string()
    }
}

impl UrlDecodable<PeerId, IllegalPeerId> for PeerId {
    fn url_decode(encoded: &str) -> Result<PeerId, IllegalPeerId> {
        PeerId::try_from(encoded)
    }
}
