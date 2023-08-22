use actix_web::{
    get,
    web::{Data, Query},
    HttpResponse,
};
use chrono::NaiveDate;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, PgPool};
use thiserror::Error;
use tracing::{error, instrument};
use uuid::Uuid;

use crate::error::{HttpError, ToHttpStatus};

#[derive(Debug, Error)]
pub enum SearchPersonsError {
    #[error("unknown error")]
    UnknownError,
}

impl ToHttpStatus for SearchPersonsError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Person {
    id: Uuid,
    name: String,
    nickname: String,
    birthdate: NaiveDate,
    stack: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetPersonsQuery {
    #[serde(rename = "t")]
    term: String,
}

#[get("/pessoas")]
pub async fn search_persons(
    query: Query<GetPersonsQuery>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, HttpError<SearchPersonsError>> {
    let pool = &**pool;

    let GetPersonsQuery { term } = query.into_inner();

    let persons = run_query(pool, &term).await?;

    Ok(HttpResponse::Ok().json(persons))
}

#[instrument]
async fn run_query(
    pool: &PgPool,
    term: &str,
) -> Result<Vec<Person>, HttpError<SearchPersonsError>> {
    query_as!(
        Person,
        "SELECT id, name, nickname, birthdate, stack FROM persons WHERE (search ILIKE CONCAT('%', $1::text, '%')) LIMIT 50",
        term,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("{:?}", e);
        HttpError::new(SearchPersonsError::UnknownError)
    })
}
