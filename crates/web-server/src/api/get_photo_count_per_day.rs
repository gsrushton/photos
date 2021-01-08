use actix_web::{get, web, HttpResponse};

#[derive(Debug, super::Error, thiserror::Error)]
pub enum Error {
    #[error("Failed to decode query string")]
    QueryStringDecodeError(#[from] serde_qs::Error),
    #[error("Database query failed")]
    DatabaseQueryError(#[from] crate::db::QueryError),
}

#[get("/photos/count-per-day")]
pub async fn endpoint(
    req: actix_web::HttpRequest,
    db: web::Data<crate::db::System>,
) -> Result<actix_web::HttpResponse, Error> {
    let params: photos_web_core::PhotoQueryParams = serde_qs::from_str(req.query_string())?;
    Ok(
        crate::db::model::Photo::count_per_day(&db, params.people.unwrap_or_default())
            .await
            .map(|day_counts| HttpResponse::Ok().json(day_counts))?,
    )
}
