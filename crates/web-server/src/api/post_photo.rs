use actix_web::{post, web, HttpResponse};

#[derive(Debug, thiserror::Error)]
pub enum SaveImageError {
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
    #[error("Operation cancelled")]
    OperationCancelled,
}

async fn save_image(
    image: image::DynamicImage,
    path: std::path::PathBuf,
) -> Result<(), SaveImageError> {
    use actix_web::error::BlockingError;
    web::block(move || image.save(path))
        .await
        .map_err(|err| match err {
            BlockingError::Error(err) => SaveImageError::ImageError(err),
            BlockingError::Canceled => SaveImageError::OperationCancelled,
        })
}

#[derive(Debug, super::Error, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse HTTP request body")]
    PayloadParsingFailed(#[from] actix_web::client::PayloadError),
    #[error("Image format '{0:?}' not persistable")]
    UnpersistableImageFormat(image::ImageFormat),
    #[error("Failed to decode image")]
    ImageLoadFailed(#[source] crate::image_ext::NewImageExtError),
    #[error("Failed to check if the photo already exists")]
    FetchExistingPhotoFailed(#[source] crate::db::QueryError),
    #[error("Photo already posted")]
    PhotoAlreadyPosted(i32),
    #[error("Failed to create the photo sub-directory")]
    CreatePhotoDirError(#[source] std::io::Error),
    #[error("Failed to create the thumb sub-directory")]
    CreateThumbDirError(#[source] std::io::Error),
    #[error("Failed to store the photo")]
    SavePhotoFailed(#[source] SaveImageError),
    #[error("Failed to store the photo's thumbnail")]
    SaveThumbFailed(#[source] SaveImageError),
    #[error("Failed to fetch known faces")]
    FetchKnownFacesFailed(#[source] crate::db::QueryError),
    #[error("Failed to setup face landmark predictor: {0}")]
    FaceLandmarkPredictorInitFailed(String),
    #[error("Failed to setup face encoder: {0}")]
    FaceEncoderInitFailed(String),
    #[error("Failed to record photo in database")]
    RecordPhotoFailed(#[source] crate::db::QueryError),
    #[error("Failed to record person in database")]
    RecordPersonFailed(#[source] crate::db::QueryError),
    #[error("Failed to record appearance in database")]
    RecordAppearanceFailed(#[source] crate::db::QueryError),
    #[error("Failed to record avatar in database")]
    RecordAvatarFailed(#[source] crate::db::QueryError),
}

const THUMB_SIZE: u32 = 256;

#[post("/photos")]
pub async fn endpoint(
    mut body: web::Payload,
    db: web::Data<crate::db::System>,
    photo_dir: web::Data<crate::api::PhotoDirPath>,
    thumb_dir: web::Data<crate::api::ThumbDirPath>,
    face_landmark_predictor_model_file_path: web::Data<
        crate::api::FaceLandmarkPredictorModelFilePath,
    >,
    face_encoder_model_file_path: web::Data<crate::api::FaceEncoderModelFilePath>,
) -> Result<actix_web::HttpResponse, Error> {
    use dlib_face_recognition::{FaceDetectorTrait, FaceEncoderTrait, LandmarkPredictorTrait};
    use futures::StreamExt;
    use image::GenericImageView;

    log::debug!("POST /photos");

    let mut bytes = web::BytesMut::new();
    while let Some(chunk) = body.next().await {
        bytes.extend_from_slice(&chunk?);
    }

    let image = crate::image_ext::ImageExt::new(std::io::Cursor::new(&bytes))
        .map_err(Error::ImageLoadFailed)?;

    let image_digest = crate::db::model::Digest::compute(image.as_bytes());
    {
        let image_digest = image_digest.clone();
        db.run_query(move |db_connection| {
            use crate::db::schema::photos;
            use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

            photos::table
                .select(photos::id)
                .filter(photos::digest.eq(image_digest))
                .limit(1)
                .load::<i32>(&db_connection)
        })
        .await
        .map_err(|err| Error::FetchExistingPhotoFailed(err))
        .and_then(|ids| match ids.as_slice() {
            [] => Ok(()),
            [id] => Err(Error::PhotoAlreadyPosted(*id)),
            _ => unreachable!(),
        })?;
    }

    let orientation = image.orientation();

    log::debug!("  ORIENTATION {:?}", orientation);

    let original_datetime = image.original_datetime();

    log::debug!("  ORIGINAL DATETIME {:?}", original_datetime);

    let image_format = image.format();

    log::debug!("  FORMAT {:?}", image_format);

    let upload_datetime = chrono::Utc::now().naive_utc();

    let photo_file_ext = match image_format.extensions_str() {
        [ext, ..] => ext,
        _ => return Err(Error::UnpersistableImageFormat(image_format)),
    };

    let photo_file_name = format!("{}.{}", image_digest, photo_file_ext);

    let month_sub_dir = std::path::PathBuf::from(format!(
        "{}",
        original_datetime.unwrap_or(upload_datetime).format("%Y-%m")
    ));

    let photo_dir = photo_dir.join(&month_sub_dir);
    std::fs::create_dir_all(&photo_dir).map_err(Error::CreatePhotoDirError)?;

    let thumb_dir = thumb_dir.join(&month_sub_dir);
    std::fs::create_dir_all(&thumb_dir).map_err(Error::CreateThumbDirError)?;

    let photo_file_path = photo_dir.join(&photo_file_name);
    let thumb_file_path = thumb_dir.join(&photo_file_name);

    log::debug!("  PHOTO PATH {:?}", photo_file_path);
    log::debug!("  THUMB PATH {:?}", thumb_file_path);

    let image = image.reorient();
    let thumb = crate::image_ext::thumbnail(&image, THUMB_SIZE);

    let (image_width, image_height) = image.dimensions();
    let (thumb_width, thumb_height) = thumb.dimensions();

    web::block(move || {
        std::io::copy(
            &mut std::io::Cursor::new(bytes),
            &mut std::io::BufWriter::new(std::fs::File::create(photo_file_path)?),
        )
    })
    .await
    .map_err(|err| match err {
        actix_web::error::BlockingError::Error(err) => {
            SaveImageError::ImageError(image::ImageError::IoError(err))
        }
        actix_web::error::BlockingError::Canceled => SaveImageError::OperationCancelled,
    })
    .map_err(|err| Error::SavePhotoFailed(err))?;

    save_image(thumb, thumb_file_path)
        .await
        .map_err(|err| Error::SaveThumbFailed(err))?;

    let photo_id = crate::db::model::Photo::insert(
        &db,
        image_digest,
        month_sub_dir
            .join(&photo_file_name)
            .to_string_lossy()
            .into_owned(),
        image_width,
        image_height,
        thumb_width,
        thumb_height,
        original_datetime,
        upload_datetime,
    )
    .await
    .map_err(|err| Error::RecordPhotoFailed(err))?;

    log::debug!("  PHOTO ID {}", photo_id);

    let image_matrix = dlib_face_recognition::ImageMatrix::from_image(&image.into_rgb8());

    let known_faces = db
        .run_query(move |db_connection| {
            use crate::db::schema::appearances;
            use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

            appearances::table
                .select((appearances::person, appearances::face_encoding))
                .filter(appearances::reference.eq(true))
                .load::<(i32, crate::db::model::FaceEncoding)>(&db_connection)
        })
        .await
        .map_err(|err| Error::FetchKnownFacesFailed(err))?;

    log::debug!("  KNOWN FACE COUNT {}", known_faces.len());

    let face_locations = dlib_face_recognition::FaceDetector::new().face_locations(&image_matrix);

    log::debug!("  FOUND FACE COUNT {}", face_locations.len());

    let mut face_landmark_predictor = dlib_face_recognition::LandmarkPredictor::new(
        face_landmark_predictor_model_file_path.as_path(),
    )
    .map_err(|err| Error::FaceLandmarkPredictorInitFailed(err))?;

    let mut face_encoder =
        dlib_face_recognition::FaceEncoderNetwork::new(face_encoder_model_file_path.as_path())
            .map_err(|err| Error::FaceEncoderInitFailed(err))?;

    for face_location in face_locations.into_iter() {
        const TOLERANCE: f64 = 0.6;

        log::debug!(
            "  FOUND FACE @ {} {} {} {}",
            face_location.top,
            face_location.left,
            face_location.bottom,
            face_location.right
        );

        let landmarks = face_landmark_predictor.face_landmarks(&image_matrix, face_location);
        if landmarks.is_empty() {
            continue;
        }

        let face_encoding = crate::db::model::FaceEncoding::from(
            face_encoder
                .get_face_encodings(&image_matrix, &[landmarks], 0)
                .get(0)
                .unwrap()
                .clone(),
        );

        let (person_id, new_person) =
            match known_faces
                .iter()
                .fold(None, |best, (known_person_id, known_face_encoding)| {
                    let distance = known_face_encoding.distance(&face_encoding);
                    if distance
                        < best
                            .map(|(_, best_distance)| best_distance)
                            .unwrap_or(TOLERANCE)
                    {
                        Some((*known_person_id, distance))
                    } else {
                        best
                    }
                }) {
                Some((person_id, _)) => {
                    log::debug!("  FOUND PERSON {}", person_id);
                    (person_id, false)
                }
                None => crate::db::model::Person::insert(&db)
                    .await
                    .map(|person_id| (person_id, true))
                    .map_err(Error::RecordPersonFailed)?,
            };

        let appearance_id = crate::db::model::Appearance::insert(
            &db,
            person_id,
            photo_id,
            new_person,
            face_location.top as i32,
            face_location.left as i32,
            face_location.bottom as i32,
            face_location.right as i32,
            face_encoding,
        )
        .await
        .map_err(Error::RecordAppearanceFailed)?;

        if new_person {
            crate::db::model::Avatar::insert(&db, person_id, appearance_id)
                .await
                .map_err(Error::RecordAvatarFailed)?;
        }
    }

    Ok(HttpResponse::Ok().finish())
}
