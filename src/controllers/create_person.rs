use actix_web::{
    http::header::LOCATION,
    post,
    web::{Data, Json},
    HttpResponse,
};
use chrono::NaiveDate;
use http::StatusCode;
use serde::Deserialize;
use sqlx::{query_as, PgPool};
use thiserror::Error;
use tracing::{error, instrument};
use uuid::Uuid;

use crate::{
    error::{HttpError, ToHttpStatus},
    utils::LengthRestrictedString,
};

const POSTGRES_UNIQUE_VALIDATION_ERROR_CODE: &str = "23505";

const MIN_NICKNAME_SIZE: usize = 1;
const MAX_NICKNAME_SIZE: usize = 32;
const MIN_NAME_SIZE: usize = 1;
const MAX_NAME_SIZE: usize = 100;
const MIN_STACK_NAME_SIZE: usize = 1;
const MAX_STACK_NAME_SIZE: usize = 32;

#[derive(Debug, Deserialize)]
pub struct CreatePersonDto {
    #[serde(rename = "apelido")]
    nickname: LengthRestrictedString<MIN_NICKNAME_SIZE, MAX_NICKNAME_SIZE>,
    #[serde(rename = "nome")]
    name: LengthRestrictedString<MIN_NAME_SIZE, MAX_NAME_SIZE>,
    #[serde(rename = "nascimento")]
    birthdate: NaiveDate,
    #[serde(rename = "stack")]
    stack: Option<Vec<LengthRestrictedString<MIN_STACK_NAME_SIZE, MAX_STACK_NAME_SIZE>>>,
}

#[derive(Debug, Deserialize)]
struct CreatePersonDbResponse {
    id: Uuid,
}

#[derive(Debug, Error)]
pub enum CreatePersonError {
    #[error("there is already a person with that nickname")]
    NicknameAlreadyExists,
    #[error("unknown error")]
    UnknownError,
}

impl ToHttpStatus for CreatePersonError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreatePersonError::NicknameAlreadyExists => StatusCode::UNPROCESSABLE_ENTITY,
            CreatePersonError::UnknownError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[post("/pessoas")]
pub async fn create_person(
    body: Json<CreatePersonDto>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, HttpError<CreatePersonError>> {
    let pool = &**pool;

    let CreatePersonDto {
        birthdate,
        name,
        nickname,
        stack,
    } = body.0;

    let stack: Vec<String> = stack
        .map(|stack| stack.iter().map(|s| s.as_ref().to_owned()).collect())
        .unwrap_or(vec![]);

    let CreatePersonDbResponse { id } = run_query(
        pool,
        name.as_ref(),
        &nickname.as_ref(),
        birthdate,
        &stack[..],
    )
    .await?;

    Ok(HttpResponse::Created()
        .insert_header((LOCATION, format!("/pessoas/{}", id)))
        .finish())
}

#[instrument]
async fn run_query(
    pool: &PgPool,
    name: &str,
    nickname: &str,
    birthdate: NaiveDate,
    stack: &[String],
) -> Result<CreatePersonDbResponse, HttpError<CreatePersonError>> {
    query_as!(
        CreatePersonDbResponse,
        "INSERT INTO persons (name, nickname, birthdate, stack) VALUES ($1, $2, $3, $4) RETURNING id",
        name,
        nickname,
        birthdate,
        stack,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!("{:?}", e);
        if let sqlx::Error::Database(error) = e {
            if let Some(code) = error.code() {
                if code == POSTGRES_UNIQUE_VALIDATION_ERROR_CODE {
                    return HttpError::new(CreatePersonError::NicknameAlreadyExists);
                }
            }
        }
        HttpError::new(CreatePersonError::UnknownError)
    })
}
