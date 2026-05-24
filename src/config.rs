use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub worker_concurrency: usize,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            port: env::var("PORT").unwrap_or_else(|_| "4000".into()).parse()?,
            worker_concurrency: env::var("WORKER_CONCURRENCY")
                .unwrap_or_else(|_| "4".into())
                .parse()?,
        })
    }
}
