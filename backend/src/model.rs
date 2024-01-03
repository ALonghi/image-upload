use aws_sdk_s3::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]

pub struct DeleteRequest {
    pub file_name: String,
}

#[derive(Debug, Serialize)]

pub struct UploadResponse {
    pub data: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]

pub struct ListResponse {
    pub data: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StandardResponse {
    pub data: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct EnvVars {
    pub region: String,
    pub bucket: String,
    pub bucket_url: String,
}

impl EnvVars {
    pub fn init() -> Self {
        let region =
            std::env::var("AWS_REGION").expect("AWS_REGION not found - region not provided");
        let bucket = std::env::var("AWS_S3_BUCKET")
            .expect("AWS_S3_BUCKET not found - bucket name not provided");
        let bucket_url = std::env::var("AWS_S3_BUCKET_URL")
            .expect("AWS_S3_BUCKET_URL not found - bucket url not provided");
        EnvVars {
            region,
            bucket,
            bucket_url,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub s3_client: Client,
    pub env_vars: EnvVars,
}
