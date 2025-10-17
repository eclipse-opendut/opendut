use crate::fs::File;
use sha2::{Digest, Sha256};
use std::io;
use std::io::Read;
use std::path::Path;

pub fn file(path: impl AsRef<Path>) -> Result<Vec<u8>, io::Error> {
    let file = File::open(path.as_ref())?;
    sha256_digest(file)
}

pub fn string(string: impl AsRef<str>) -> Result<Vec<u8>, io::Error> {
    let bytes = string.as_ref().as_bytes();
    sha256_digest(bytes)
}

fn sha256_digest(mut reader: impl Read) -> Result<Vec<u8>, io::Error> {
    let mut hasher = Sha256::new();
    let _ = io::copy(&mut reader, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(hash.to_vec())
}
