mod config;
mod models;
mod routes;
mod scraper;

use axum::{
    routing::{get, post},
    Json, Router,
};
use config::Config;
use models::Selector;
use reqwest::Client;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env()?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let http_client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let semaphore = Arc::new(Semaphore::new(config.worker_concurrency));
    let pool_clone = pool.clone();
    let client_clone = http_client.clone();
    let sem_clone = semaphore.clone();

    tokio::spawn(async move {
        run_worker(pool_clone, client_clone, sem_clone).await;
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/jobs", post(routes::jobs::create_job))
        .route("/api/v1/jobs", get(routes::jobs::list_jobs))
        .route("/api/v1/jobs/:id", get(routes::jobs::get_job))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let addr = format!("0.0.0.0:{}", config.port);
    info!("DataHarvest listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({"status": "healthy"}))
}

async fn run_worker(pool: sqlx::PgPool, client: Client, semaphore: Arc<Semaphore>) {
    loop {
        let result = process_pending_jobs(&pool, &client, &semaphore).await;
        if let Err(e) = result {
            tracing::error!("worker error: {}", e);
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

async fn process_pending_jobs(
    pool: &sqlx::PgPool,
    client: &Client,
    semaphore: &Arc<Semaphore>,
) -> anyhow::Result<()> {
    let jobs = sqlx::query_as::<_, models::Job>(
        "UPDATE jobs SET status='running' WHERE id IN (
            SELECT id FROM jobs WHERE status='pending' LIMIT 10
        ) RETURNING *",
    )
    .fetch_all(pool)
    .await?;

    let mut handles = vec![];

    for job in jobs {
        let pool = pool.clone();
        let client = client.clone();
        let sem = semaphore.clone();
        let job_id: Uuid = job.id;
        let url = job.url.clone();
        let selectors: Vec<Selector> =
            serde_json::from_value(job.selectors.clone()).unwrap_or_default();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            match scraper::fetch_and_extract(&client, &url, &selectors).await {
                Ok(data) => {
                    let _ = sqlx::query(
                        "INSERT INTO scraped_results (job_id, data) VALUES ($1, $2)",
                    )
                    .bind(job_id)
                    .bind(&data)
                    .execute(&pool)
                    .await;

                    let _ = sqlx::query(
                        "UPDATE jobs SET status='completed', finished_at=now() WHERE id=$1",
                    )
                    .bind(job_id)
                    .execute(&pool)
                    .await;

                    info!("job {} completed", job_id);
                }
                Err(e) => {
                    let msg = e.to_string();
                    let _ = sqlx::query(
                        "UPDATE jobs SET status='failed', error_msg=$1, finished_at=now() WHERE id=$2",
                    )
                    .bind(&msg)
                    .bind(job_id)
                    .execute(&pool)
                    .await;

                    tracing::error!("job {} failed: {}", job_id, msg);
                }
            }
        });
        handles.push(handle);
    }

    for h in handles {
        let _ = h.await;
    }

    Ok(())
}
