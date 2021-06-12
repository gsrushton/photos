use actix_web::{get, web};

#[derive(Debug, super::Error, thiserror::Error)]
pub enum Error {
    #[error("Failed to decode query string")]
    QueryStringDecodeError(#[from] serde_qs::Error),
    #[error("Database query failed")]
    DatabaseQueryError(#[from] crate::db::QueryError),
    #[error("No matching person")]
    NotFound,
    #[error("Failed to generate image")]
    ImageError(#[source] crate::db::model::avatar::ImageError),
    #[error("Failed to encode avatar")]
    EncodeError(#[source] image::ImageError),
}

#[get("/people/{id:\\d+}/avatar")]
pub async fn endpoint(
    req: actix_web::HttpRequest,
    person_id: web::Path<i32>,
    photo_dir: web::Data<crate::api::PhotoDirPath>,
    db: web::Data<crate::db::System>,
) -> Result<actix_web::HttpResponse, Error> {
    let params: photos_web_core::AvatarQueryParams = serde_qs::from_str(req.query_string())?;

    crate::db::model::Avatar::fetch_for_person(&db, *person_id)
        .await?
        .ok_or(Error::NotFound)
        .and_then(|avatar| {
            avatar
                .image(params.size.unwrap_or(128), &photo_dir.into_inner())
                .map_err(Error::ImageError)
        })
        .and_then(|image| crate::image_ext::encode_image(&image).map_err(Error::EncodeError))
}
