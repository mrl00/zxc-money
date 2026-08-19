use async_trait::async_trait;
use thiserror::Error;

/// Errors from the file storage infrastructure.
#[derive(Debug, Error)]
pub enum StorageError {
    /// The file could not be written.
    #[error("write failed: {0}")]
    WriteFailed(String),

    /// The file could not be read.
    #[error("read failed: {0}")]
    ReadFailed(String),

    /// The file was not found.
    #[error("not found: {0}")]
    NotFound(String),
}

/// Port for persisting and retrieving binary files.
///
/// Frontends must implement this trait to handle file I/O — exporting
/// reports (CSV, PDF), importing statements, or storing backups.
///
/// # Example
///
/// A TUI frontend might use the local filesystem:
/// ```ignore
/// struct LocalFs;
///
/// #[async_trait]
/// impl FileStorage for LocalFs {
///     async fn save(&self, path: &str, data: &[u8]) -> Result<(), StorageError> {
///         tokio::fs::write(path, data).await.map_err(|e| StorageError::WriteFailed(e.to_string()))
///     }
///
///     async fn load(&self, path: &str) -> Result<Vec<u8>, StorageError> {
///         tokio::fs::read(path).await.map_err(|e| StorageError::ReadFailed(e.to_string()))
///     }
/// }
/// ```
#[async_trait]
pub trait FileStorage: Send + Sync {
    /// Saves binary data to the given path.
    async fn save(&self, path: &str, data: &[u8]) -> Result<(), StorageError>;

    /// Loads binary data from the given path.
    async fn load(&self, path: &str) -> Result<Vec<u8>, StorageError>;
}

/// Mock file storage for testing.
///
/// Stores files in an in-memory `HashMap`.
pub struct MockFileStorage {
    files: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
}

impl MockFileStorage {
    /// Creates a new empty mock.
    pub fn new() -> Self {
        Self {
            files: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Returns a copy of all stored files.
    pub fn files(&self) -> std::collections::HashMap<String, Vec<u8>> {
        self.files.lock().unwrap().clone()
    }
}

impl Default for MockFileStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileStorage for MockFileStorage {
    async fn save(&self, path: &str, data: &[u8]) -> Result<(), StorageError> {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), data.to_vec());
        Ok(())
    }

    async fn load(&self, path: &str) -> Result<Vec<u8>, StorageError> {
        self.files
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(path.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_save_and_load() {
        let mock = MockFileStorage::new();
        mock.save("report.csv", b"date,amount").await.unwrap();

        let data = mock.load("report.csv").await.unwrap();
        assert_eq!(data, b"date,amount");
    }

    #[tokio::test]
    async fn test_mock_load_not_found() {
        let mock = MockFileStorage::new();
        let result = mock.load("missing.csv").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_overwrite() {
        let mock = MockFileStorage::new();
        mock.save("f.txt", b"v1").await.unwrap();
        mock.save("f.txt", b"v2").await.unwrap();

        let data = mock.load("f.txt").await.unwrap();
        assert_eq!(data, b"v2");
        assert_eq!(mock.files().len(), 1);
    }
}
