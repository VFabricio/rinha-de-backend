use actix_web::{
    get,
    web::{Data, Path},
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
pub enum GetPersonError {
    #[error("person not found")]
    PersonNotFound,
    #[error("unknown error")]
    UnknownError,
}

impl ToHttpStatus for GetPersonError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetPersonError::PersonNotFound => StatusCode::NOT_FOUND,
            GetPersonError::UnknownError => StatusCode::INTERNAL_SERVER_ERROR,
        }
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

#[get("/pessoas/{id}")]
pub async fn get_person(
    path: Path<Uuid>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, HttpError<GetPersonError>> {
    let pool = &**pool;

    let id = path.into_inner();

    let person = run_query(pool, id).await?;

    Ok(HttpResponse::Ok().json(person))
}

#[instrument]
async fn run_query(pool: &PgPool, id: Uuid) -> Result<Person, HttpError<GetPersonError>> {
    query_as!(
        Person,
        "SELECT id, name, nickname, birthdate, stack FROM persons WHERE id = $1",
        id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!("{:?}", e);
        HttpError::new(GetPersonError::UnknownError)
    })?
    .ok_or(HttpError::new(GetPersonError::PersonNotFound))
}
