use serde::{ Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ErrorCode {
    Unauthorized,
    InvalidToken,
    TokenExpired,
    InvalidCredentials,
    Forbidden,
    AccountInactive,
    SessionExpired,
    
    // Resources
    NotFound,
    UserNotFound,
    RoomNotFound,
    TopicNotFound,

    // Validation
    ValidationError,    
    InvalidInput,
    MissingField,

    // Conflicts
    AlreadyExists,
    UsernameTaken,
    EmailTaken,    
    Conflict,
    
    // Business Logic
    RoomFull,
    PermissionDenied,
    RateLimitExceeded,
    
    // System
    DatabaseError,
    InternalError,    
    ServiceUnavailable,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}