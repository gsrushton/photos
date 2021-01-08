use actix_web::{get, web, HttpResponse};

#[derive(Debug, super::Error, thiserror::Error)]
pub enum Error {
    #[error("Database query failed")]
    DatabaseQueryError(#[from] crate::db::QueryError),
}

#[get("/photos/{id:\\d+}/appearances")]
pub async fn endpoint(
    photo_id: web::Path<i32>,
    db: web::Data<crate::db::System>,
) -> Result<actix_web::HttpResponse, Error> {
    Ok(
        crate::db::model::Appearance::fetch_all_for_photo(&db, *photo_id)
            .await
            .map(|appearances| {
                HttpResponse::Ok().json(photos_web_core::Appearances::from(appearances))
            })?,
    )
}
