use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Selector {
    pub field: String,
    pub css: String,
    pub attribute: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Job {
    pub id: Uuid,
    pub url: String,
    pub selectors: serde_json::Value,
    pub status: String,
    pub error_msg: Option<String>,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ScrapedResult {
    pub id: Uuid,
    pub job_id: Uuid,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub url: String,
    pub selectors: Vec<Selector>,
}

#[derive(Debug, Serialize)]
pub struct JobResponse {
    pub id: Uuid,
    pub url: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error_msg: Option<String>,
}

impl From<Job> for JobResponse {
    fn from(j: Job) -> Self {
        Self {
            id: j.id,
            url: j.url,
            status: j.status,
            created_at: j.created_at,
            finished_at: j.finished_at,
            error_msg: j.error_msg,
        }
    }
}
