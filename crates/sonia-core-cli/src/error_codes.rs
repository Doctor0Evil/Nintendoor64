use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SoniaErrorCode {
    GenericInvalidInput,
    ArtifactDecodeError,
    ArtifactWriteFailed,
    UnknownCommand,
    SessionNotFound,
    SessionWriteFailed,
    ListFailed,
}

impl SoniaErrorCode {
    pub fn exit_code(self) -> i32 {
        match self {
            SoniaErrorCode::GenericInvalidInput => 1,
            SoniaErrorCode::ArtifactDecodeError => 2,
            SoniaErrorCode::ArtifactWriteFailed => 3,
            SoniaErrorCode::UnknownCommand => 1,
            SoniaErrorCode::SessionNotFound => 1,
            SoniaErrorCode::SessionWriteFailed => 1,
            SoniaErrorCode::ListFailed => 1,
        }
    }
}
