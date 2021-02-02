use actix_web::{get, web, HttpResponse};

#[derive(Debug, super::Error, thiserror::Error)]
pub enum Error {
    #[error("Database query failed")]
    DatabaseQueryError(#[from] crate::db::QueryError),
}

#[get("/photo/{id:\\d+}")]
pub async fn endpoint(
    photo_id: web::Path<i32>,
    db: web::Data<crate::db::System>,
) -> Result<actix_web::HttpResponse, Error> {
    Ok(crate::db::model::Photo::fetch(&db, *photo_id)
        .await
        .map(|photo| {
            HttpResponse::Ok().json(photo.map(|photo| -> photos_web_core::Photo { photo.into() }))
        })?)
}
