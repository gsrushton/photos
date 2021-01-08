use actix_web::{get, web, HttpResponse};

#[derive(Debug, super::Error, thiserror::Error)]
pub enum Error {
    #[error("Database query failed")]
    DatabaseQueryError(#[from] crate::db::QueryError),
}

#[get("/people/{id:\\d+}")]
pub async fn endpoint(
    person_id: web::Path<i32>,
    db: web::Data<crate::db::System>,
) -> Result<actix_web::HttpResponse, Error> {
    Ok(crate::db::model::Person::fetch(&db, *person_id)
        .await
        .map(|person| {
            HttpResponse::Ok()
                .json(person.map(|person| -> photos_web_core::Person { person.into() }))
        })?)
}
