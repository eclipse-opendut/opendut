use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use crate::fs;

pub fn create_file_and_ensure_it_can_only_be_read_or_modified_by_owner(file_path: impl AsRef<Path>) -> anyhow::Result<()> {
    if !file_path.as_ref().exists() {
        fs::File::create(file_path.as_ref())?;
    }
    let read_only_permissions = std::fs::Permissions::from_mode(0o600);
    fs::set_permissions(file_path, read_only_permissions)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_readonly_file() {
        // Arrange: assumes to be run in unix
        // 100 means it's a regular file
        // 600 means the owner may read and write that file and nobody else
        let expected_permissions = std::fs::Permissions::from_mode(0o100600 );
        let temp_dir = tempfile::TempDir::new().expect("failed to create temp dir");
        let path = temp_dir.path().join("edgar.toml");
        assert!(!path.exists());

        // Act: create file
        create_file_and_ensure_it_can_only_be_read_or_modified_by_owner(&path).unwrap();

        // Assert:
        let file = fs::File::open(path).expect("Expected to open file");
        let permissions = file.metadata().expect("Expected to retrieve file metadata.").permissions();
        assert_eq!(permissions, expected_permissions);
    }
}
