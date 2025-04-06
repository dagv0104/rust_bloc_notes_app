use actix_web::{HttpResponse, ResponseError};
use diesel::result::Error as DieselError;
use std::fmt;

#[derive(Debug)]
pub enum ServiceError {
    Unauthorized(String),
    InternalServerError(String),
    BadRequest(String),
    Forbidden(String),
    NotFound(String),
}

impl ServiceError {
    pub fn unauthorized(msg: &str) -> Self {
        ServiceError::Unauthorized(msg.to_string())
    }
    
    pub fn internal_server_error(msg: &str) -> Self {
        ServiceError::InternalServerError(msg.to_string())
    }
    
    pub fn bad_request(msg: &str) -> Self {
        ServiceError::BadRequest(msg.to_string())
    }
    
    pub fn forbidden(msg: &str) -> Self {
        ServiceError::Forbidden(msg.to_string())
    }
    
    pub fn not_found(msg: &str) -> Self {
        ServiceError::NotFound(msg.to_string())
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ServiceError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
            ServiceError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ServiceError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            ServiceError::NotFound(msg) => write!(f, "Not Found: {}", msg),
        }
    }
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::Unauthorized(msg) => HttpResponse::Unauthorized().json(msg),
            ServiceError::InternalServerError(msg) => HttpResponse::InternalServerError().json(msg),
            ServiceError::BadRequest(msg) => HttpResponse::BadRequest().json(msg),
            ServiceError::Forbidden(msg) => HttpResponse::Forbidden().json(msg),
            ServiceError::NotFound(msg) => HttpResponse::NotFound().json(msg),
        }
    }
}

impl From<DieselError> for ServiceError {
    fn from(error: DieselError) -> ServiceError {
        match error {
            DieselError::NotFound => ServiceError::not_found("Resource not found"),
            _ => ServiceError::internal_server_error(&format!("Database error: {}", error)),
        }
    }
}