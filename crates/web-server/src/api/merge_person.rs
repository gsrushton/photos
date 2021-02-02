use actix_web::{post, web, HttpResponse};

#[derive(Debug, super::Error, thiserror::Error)]
pub enum Error {
    #[error("Database update failed")]
    DatabaseUpdateQueryError(#[from] crate::db::UpdateQueryError),
}

#[post("/people/{dst_id:\\d+}/merge/{src_id:\\d+}")]
pub async fn endpoint(
    web::Path((dst_id, src_id)): web::Path<(i32, i32)>,
    db: web::Data<crate::db::System>,
) -> Result<actix_web::HttpResponse, Error> {
    Ok(crate::db::model::Person::merge(&db, dst_id, src_id)
        .await
        .map(|_| HttpResponse::Ok().json(()))?)
}
