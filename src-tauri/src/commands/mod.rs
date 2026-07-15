pub mod sts;
pub mod s3;
pub mod cloudformation;
pub mod billing;
pub mod dashboard;

// Re-export shared types
use serde::Serialize;

#[derive(Serialize)]
pub struct GenericResponse {
    pub success: bool,
    pub error_message: Option<String>,
}
