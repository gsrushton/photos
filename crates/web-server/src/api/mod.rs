mod get_appearance_avatar;
mod get_people;
mod get_person;
mod get_person_avatar;
mod get_photo_appearances;
mod get_photo_count_per_day;
mod get_photos_for_day;
mod merge_person;
mod post_photo;
mod put_person;

pub use photos_web_server_derive::ApiError as Error;

pub struct PhotoDirPath(std::path::PathBuf);

impl From<std::path::PathBuf> for PhotoDirPath {
    fn from(path: std::path::PathBuf) -> Self {
        Self(path)
    }
}

impl std::ops::Deref for PhotoDirPath {
    type Target = std::path::Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ThumbDirPath(std::path::PathBuf);

impl From<std::path::PathBuf> for ThumbDirPath {
    fn from(path: std::path::PathBuf) -> Self {
        Self(path)
    }
}

impl std::ops::Deref for ThumbDirPath {
    type Target = std::path::Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct FaceLandmarkPredictorModelFilePath(std::path::PathBuf);

impl FaceLandmarkPredictorModelFilePath {
    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }
}

impl From<std::path::PathBuf> for FaceLandmarkPredictorModelFilePath {
    fn from(path: std::path::PathBuf) -> Self {
        Self(path)
    }
}

impl std::ops::Deref for FaceLandmarkPredictorModelFilePath {
    type Target = std::path::Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct FaceEncoderModelFilePath(std::path::PathBuf);

impl FaceEncoderModelFilePath {
    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }
}

impl From<std::path::PathBuf> for FaceEncoderModelFilePath {
    fn from(path: std::path::PathBuf) -> Self {
        Self(path)
    }
}

impl std::ops::Deref for FaceEncoderModelFilePath {
    type Target = std::path::Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_appearance_avatar::endpoint)
        .service(get_person_avatar::endpoint)
        .service(get_people::endpoint)
        .service(get_person::endpoint)
        .service(get_photo_appearances::endpoint)
        .service(get_photo_count_per_day::endpoint)
        .service(get_photos_for_day::endpoint)
        .service(merge_person::endpoint)
        .service(post_photo::endpoint)
        .service(put_person::endpoint);
}
