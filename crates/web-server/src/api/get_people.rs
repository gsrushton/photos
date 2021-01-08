use actix_web::{get, web, HttpResponse};

#[derive(Debug, super::Error, thiserror::Error)]
enum Error {
    #[error("Database query failed")]
    DatabaseQueryError(#[from] crate::db::QueryError),
}

#[get("/people")]
async fn endpoint(db: web::Data<crate::db::System>) -> Result<actix_web::HttpResponse, Error> {
    Ok(crate::db::model::Person::fetch_all(&db)
        .await
        .map(|people| HttpResponse::Ok().json(photos_web_core::People::from(people)))?)
}
