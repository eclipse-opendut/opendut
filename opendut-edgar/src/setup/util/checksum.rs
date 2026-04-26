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
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;
    hasher.update(&mut data);
    let hash = hasher.finalize();
    Ok(hash.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_provide_sha256_digest() {
        let data = io::Cursor::new("hello");

        let result = sha256_digest(data).unwrap();

        let hex_string = result.iter()
            .map(|byte| format!("{:02x}", byte)) //format bytes as hex string
            .collect::<String>();
        assert_eq!(
            hex_string,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824" //output from `echo -n "hello" | sha256sum`
        );
    }
}
