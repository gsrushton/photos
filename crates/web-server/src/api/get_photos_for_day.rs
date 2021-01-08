use actix_web::{get, web, HttpResponse};

#[derive(Debug, super::Error, thiserror::Error)]
pub enum Error {
    #[error("Failed to decode query string")]
    QueryStringDecodeError(#[from] serde_qs::Error),
    #[error("Database query failed")]
    DatabaseQueryError(#[from] crate::db::QueryError),
}

#[get("/photos/for-day/{date}")]
pub async fn endpoint(
    req: actix_web::HttpRequest,
    date: web::Path<chrono::NaiveDate>,
    db: web::Data<crate::db::System>,
) -> Result<HttpResponse, Error> {
    let params: photos_web_core::PhotoQueryParams = serde_qs::from_str(req.query_string())?;
    Ok(
        crate::db::model::Photo::fetch_all_for_day(&db, *date, params.people.unwrap_or_default())
            .await
            .map(|photos| HttpResponse::Ok().json(photos_web_core::Photos::from(photos)))?,
    )
}
