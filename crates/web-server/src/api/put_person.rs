use actix_web::{put, web, HttpResponse};

#[derive(Debug, super::Error, thiserror::Error)]
pub enum Error {
    #[error("Database update failed")]
    DatabaseUpdateQueryError(#[from] crate::db::UpdateQueryError),
}

#[put("/people/{id:\\d+}")]
pub async fn endpoint(
    person_id: web::Path<i32>,
    person: web::Json<photos_web_core::Person>,
    db: web::Data<crate::db::System>,
) -> Result<actix_web::HttpResponse, Error> {
    Ok(crate::db::model::Person::from(person.into_inner())
        .record(&db, *person_id)
        .await
        .map(|_| HttpResponse::Ok().finish())?)
}
