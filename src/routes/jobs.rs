use crate::models::{CreateJobRequest, Job, JobResponse, ScrapedResult};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_job(
    State(pool): State<PgPool>,
    Json(body): Json<CreateJobRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let selectors = serde_json::to_value(&body.selectors).map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    let job = sqlx::query_as::<_, Job>(
        "INSERT INTO jobs (url, selectors) VALUES ($1, $2) RETURNING *",
    )
    .bind(&body.url)
    .bind(&selectors)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    Ok((StatusCode::CREATED, Json(json!(JobResponse::from(job)))))
}

pub async fn list_jobs(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let jobs = sqlx::query_as::<_, Job>("SELECT * FROM jobs ORDER BY created_at DESC LIMIT 100")
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    let responses: Vec<JobResponse> = jobs.into_iter().map(JobResponse::from).collect();
    Ok(Json(json!({"data": responses, "total": responses.len()})))
}

pub async fn get_job(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "job not found"})),
            )
        })?;

    let results = sqlx::query_as::<_, ScrapedResult>(
        "SELECT * FROM scraped_results WHERE job_id = $1 ORDER BY created_at DESC",
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    Ok(Json(json!({
        "job": JobResponse::from(job),
        "results": results.iter().map(|r| &r.data).collect::<Vec<_>>(),
    })))
}
