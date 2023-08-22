use actix_web::{get, http::header::ContentType, web::Data, HttpResponse};
use http::StatusCode;
use sqlx::{query_as, PgPool};
use thiserror::Error;
use tracing::error;

use crate::error::{HttpError, ToHttpStatus};

#[derive(Debug, Error)]
pub enum CountPersonsError {
    #[error("unknown error")]
    UnknownError,
}

impl ToHttpStatus for CountPersonsError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub struct CountPersonsDbResponse {
    count: i64,
}

#[get("/contagem-pessoas")]
pub async fn count_persons(
    pool: Data<PgPool>,
) -> Result<HttpResponse, HttpError<CountPersonsError>> {
    let pool = &**pool;

    let CountPersonsDbResponse { count } = query_as!(
        CountPersonsDbResponse,
        r#"SELECT COUNT(*) AS "count!" FROM persons;"#
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!("{:?}", e);
        HttpError::new(CountPersonsError::UnknownError)
    })?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(count.to_string()))
}
