mod err;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use err::ShortenError;
use http::{header::LOCATION, HeaderMap, StatusCode};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

const SERVER_ADDR: &str = "0.0.0.0:9090";
const MYSQL_URL: &str = "mysql://root:root@localhost:3306/shortener";

#[derive(Debug, Clone)]
struct AppState {
    mysql: MySqlPool,
}

#[derive(Debug, Deserialize)]
struct ShortenReq {
    url: String,
}

#[derive(Debug, Serialize)]
struct ShortenRes {
    url: String,
}

#[derive(Debug, FromRow)]
struct UrlRecord {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let state = AppState::try_new(MYSQL_URL).await?;
    info!("MySQL connected");

    let listener = TcpListener::bind(SERVER_ADDR).await?;
    info!("Server listening on {}", SERVER_ADDR);

    let app = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(state);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn shorten(
    State(state): State<AppState>,
    Json(req): Json<ShortenReq>,
) -> anyhow::Result<impl IntoResponse, ShortenError> {
    let url = state.shorten(&req.url).await?;
    Ok((StatusCode::CREATED, Json(ShortenRes { url })))
}

async fn redirect(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> anyhow::Result<impl IntoResponse, ShortenError> {
    let url = state.get_url(&id).await?;

    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.parse().unwrap());
    Ok((StatusCode::PERMANENT_REDIRECT, headers))
}

impl AppState {
    async fn try_new(url: &str) -> Result<Self, ShortenError> {
        let pool = MySqlPool::connect(url).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS urls (
                id VARCHAR(6) PRIMARY KEY,
                url TEXT NOT NULL,
                UNIQUE KEY (url(255))
            )
        "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { mysql: pool })
    }

    async fn shorten(&self, url: &str) -> Result<String, ShortenError> {
        let transaction = self.mysql.begin().await?;
        loop {
            // for test duplicated id
            let id = nanoid!(1);
            // let id = nanoid!(6);

            let ret = self.insert_or_select(url, id.clone()).await;
            match ret {
                Ok(v) => {
                    transaction.commit().await?;
                    return Ok(v);
                }
                Err(e) => {
                    if let ShortenError::DuplicateId(_) = e {
                        info!("Duplicate id: {}, retry", id);
                        continue;
                    }
                    transaction.rollback().await?;
                    return Err(e);
                }
            }
        }
    }

    async fn insert_or_select(&self, url: &str, id: String) -> Result<String, ShortenError> {
        let ret = sqlx::query(
            r#"
            INSERT INTO urls (id, url) VALUES (?, ?)
            "#,
        )
        .bind(&id)
        .bind(url)
        .execute(&self.mysql)
        .await;

        match ret {
            Ok(_) => Ok(format!("http://{}/{}", SERVER_ADDR, id)),
            Err(e) => {
                if !e.to_string().contains("Duplicate") {
                    return Err(e.into());
                }

                warn!("{}", e);

                // Duplicate url, try to get the existed one
                if e.to_string().contains("urls.url") {
                    let ret: Result<UrlRecord, sqlx::Error> =
                        sqlx::query_as("SELECT id, url FROM urls WHERE url = ?")
                            .bind(url)
                            .fetch_one(&self.mysql)
                            .await;
                    match ret {
                        Ok(v) => Ok(format!("http://{}/{}", SERVER_ADDR, v.id)),
                        Err(e) => Err(e.into()),
                    }
                } else if e.to_string().contains("urls.PRIMARY") {
                    // Sometimes maybe both urls.id and urls.url are duplicated,
                    // but here we just take both as duplicated id,
                    // and just retry to generate a new id.
                    Err(ShortenError::DuplicateId(id.to_string()))
                } else {
                    Err(e.into())
                }
            }
        }
    }

    async fn get_url(&self, id: &str) -> Result<String, ShortenError> {
        let ret: UrlRecord = sqlx::query_as("SELECT url FROM urls WHERE id = ?")
            .bind(id)
            .fetch_one(&self.mysql)
            .await?;
        Ok(ret.url)
    }
}
