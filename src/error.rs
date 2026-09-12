//! Stable errors shared by runtime, IPC and CLI. Messages are diagnostic text,
//! never a machine-readable discriminator.
use crate::{
    runtime::{ProcessError, ProcessErrorCode},
    storage::StorageError,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidInput,
    Forbidden,
    NotFound,
    AlreadyExists,
    OperationBusy,
    PortInUse,
    PrerequisiteUnavailable,
    ProcessUnavailable,
    ProcessFailed,
    TimedOut,
    Cancelled,
    RollbackFailed,
    StorageIo,
    StorageCorrupt,
    MigrationRefused,
    UnsupportedOperation,
    UnsafePath,
    Internal,
    BrowserOpenFailed,
    WindowOpenFailed,
    ShortcutFailed,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
}
pub type AppResult<T> = Result<T, AppError>;
impl AppError {
    pub fn new(code: ErrorCode, message: impl std::fmt::Display) -> Self {
        Self {
            code,
            message: crate::runtime::redact(&message.to_string()),
        }
    }
    pub fn rollback(original: impl std::fmt::Display, cleanup: impl std::fmt::Display) -> Self {
        Self::new(ErrorCode::RollbackFailed, format!("{original} Cleanup also failed: {cleanup}. Setup may be incomplete; inspect the app files and containers before retrying."))
    }
    pub fn internal(error: impl std::fmt::Display) -> Self {
        Self::new(ErrorCode::Internal, error)
    }
    pub fn invalid(error: impl std::fmt::Display) -> Self {
        Self::new(ErrorCode::InvalidInput, error)
    }
}
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for AppError {}
impl From<ProcessError> for AppError {
    fn from(error: ProcessError) -> Self {
        let code = match error.code {
            ProcessErrorCode::ProcessUnavailable => ErrorCode::ProcessUnavailable,
            ProcessErrorCode::ProcessFailed => ErrorCode::ProcessFailed,
            ProcessErrorCode::TimedOut => ErrorCode::TimedOut,
            ProcessErrorCode::Cancelled => ErrorCode::Cancelled,
        };
        Self::new(code, error.message)
    }
}
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::new(ErrorCode::StorageIo, error)
    }
}
impl From<StorageError> for AppError {
    fn from(error: StorageError) -> Self {
        let code = match &error {
            StorageError::AlreadyExists(_) => ErrorCode::AlreadyExists,
            StorageError::NotFound(_) => ErrorCode::NotFound,
            StorageError::Io(_) => ErrorCode::StorageIo,
            StorageError::Json(_) | StorageError::InvalidRegistryVersion(_) => {
                ErrorCode::StorageCorrupt
            }
            StorageError::MigrationRefused(_) => ErrorCode::MigrationRefused,
            StorageError::InvalidComposeValue(_) => ErrorCode::InvalidInput,
            // Same meaning the launcher already shows for a busy app: wait and
            // try again, rather than a storage fault the user should act on.
            StorageError::LockUnavailable(_) => ErrorCode::OperationBusy,
        };
        Self::new(code, error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn storage_variants_and_process_codes_survive_the_command_boundary() {
        let cases = [
            (
                StorageError::AlreadyExists("duplicate".into()),
                ErrorCode::AlreadyExists,
            ),
            (
                StorageError::NotFound("missing".into()),
                ErrorCode::NotFound,
            ),
            (
                StorageError::Io(std::io::Error::other("read failed")),
                ErrorCode::StorageIo,
            ),
            (
                StorageError::InvalidRegistryVersion(99),
                ErrorCode::StorageCorrupt,
            ),
            (
                StorageError::MigrationRefused("refused".into()),
                ErrorCode::MigrationRefused,
            ),
        ];
        for (source, code) in cases {
            assert_eq!(AppError::from(source).code, code);
        }
        for (source, code) in [
            (ProcessErrorCode::Cancelled, ErrorCode::Cancelled),
            (ProcessErrorCode::TimedOut, ErrorCode::TimedOut),
            (ProcessErrorCode::ProcessFailed, ErrorCode::ProcessFailed),
            (
                ProcessErrorCode::ProcessUnavailable,
                ErrorCode::ProcessUnavailable,
            ),
        ] {
            let error = AppError::from(ProcessError::new(
                source,
                "https://user:secret@example.com failed",
            ));
            assert_eq!(error.code, code);
            assert!(!serde_json::to_string(&error).unwrap().contains("secret"));
        }
    }
}
