# DataHarvest

High-performance concurrent web scraper built in Rust. Submit scraping jobs via REST API with CSS selectors, process them asynchronously through a built-in worker pool, and retrieve structured JSON results from PostgreSQL.

---

## How it works

```
POST /api/v1/jobs  (url + CSS selectors)
        ↓
PostgreSQL — job persisted (status: pending)
        ↓
Tokio Worker Pool — picks up pending jobs
        ↓
reqwest — fetches page with connection pooling
        ↓
scraper — extracts fields via CSS selectors
        ↓
PostgreSQL — results stored as JSONB
        ↓
GET /api/v1/jobs/:id — retrieve structured data
```

---

## Tech Stack

- **Rust 1.78** + **Tokio** async runtime
- **Axum** — ergonomic async HTTP framework
- **reqwest** — async HTTP client with connection pooling
- **scraper** — HTML parsing with CSS selector support
- **sqlx** — compile-time checked async PostgreSQL queries
- **PostgreSQL 16** — JSONB result storage
- **Docker** + **Docker Compose**

---

## Getting Started

```bash
git clone https://github.com/ElitonScos/dataharvest.git
cd dataharvest

cp .env.example .env

docker compose up -d
```

API available at `http://localhost:4000`

---

## Environment Variables

```env
DATABASE_URL=postgresql://dhuser:dhpass@db:5432/dataharvest
PORT=4000
WORKER_CONCURRENCY=4
RUST_LOG=info
```

---

## API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| POST | `/api/v1/jobs` | Submit a scraping job |
| GET | `/api/v1/jobs` | List all jobs |
| GET | `/api/v1/jobs/:id` | Get job status and results |

---

## Usage example

Submit a job:
```json
POST /api/v1/jobs
{
  "url": "https://news.ycombinator.com",
  "selectors": [
    { "field": "titles", "css": ".titleline > a" },
    { "field": "scores", "css": ".score" }
  ]
}
```

Get results:
```json
GET /api/v1/jobs/{id}

{
  "job": { "id": "...", "status": "completed", "url": "..." },
  "results": [
    {
      "titles": ["Show HN: DataHarvest", "Ask HN: ..."],
      "scores": ["142 points", "89 points"]
    }
  ]
}
```

---

## Project Structure

```
dataharvest/
├── src/
│   ├── main.rs          — Axum server, Tokio worker loop, wiring
│   ├── config.rs        — env-based configuration
│   ├── models.rs        — domain types, request/response structs
│   ├── scraper.rs       — HTML fetching and CSS extraction
│   └── routes/
│       └── jobs.rs      — job endpoints
├── migrations/
│   └── 001_init.sql
├── docker/
│   └── Dockerfile       — multi-stage: rust builder → debian slim
├── docker-compose.yml
└── .env.example
```
