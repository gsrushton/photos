#![recursion_limit = "512"]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod api;
mod db;
mod image_ext;

embed_migrations!();

#[derive(Debug, thiserror::Error)]
enum ServerError {
    #[error("Failed to connect to database")]
    DatabaseInitError(#[from] db::NewSystemError),
    #[error("Failed to bind listen socket")]
    BindError(#[source] std::io::Error),
    #[error("Failed to run server")]
    RunError(#[source] std::io::Error),
}

struct StaticDirPath(std::path::PathBuf);

impl From<std::path::PathBuf> for StaticDirPath {
    fn from(path: std::path::PathBuf) -> Self {
        Self(path)
    }
}

impl std::ops::Deref for StaticDirPath {
    type Target = std::path::Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

async fn index(
    static_dir_path: actix_web::web::Data<StaticDirPath>,
) -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open(
        static_dir_path.join("index.html"),
    )?)
}

#[actix_web::get("/static/photos/{month}/{file_name}")]
async fn get_photo(
    actix_web::web::Path((month, file_name)): actix_web::web::Path<(String, String)>,
    photo_dir_path: actix_web::web::Data<api::PhotoDirPath>,
) -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open(
        photo_dir_path.join(month).join(file_name),
    )?)
}

#[actix_web::get("/static/thumbs/{month}/{file_name}")]
async fn get_thumb(
    actix_web::web::Path((month, file_name)): actix_web::web::Path<(String, String)>,
    thumb_dir_path: actix_web::web::Data<api::ThumbDirPath>,
) -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open(
        thumb_dir_path.join(month).join(file_name),
    )?)
}

async fn run(
    db_file_path: std::path::PathBuf,
    photo_file_path: std::path::PathBuf,
    thumb_file_path: std::path::PathBuf,
    static_dir_path: std::path::PathBuf,
    face_landmark_predictor_model_file_path: std::path::PathBuf,
    face_encoder_model_file_path: std::path::PathBuf,
    host: &str,
    port: u16,
) -> Result<(), ServerError> {
    let db = db::System::new(&db_file_path)?;

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(db.clone())
            .data(StaticDirPath::from(static_dir_path.clone()))
            .data(api::PhotoDirPath::from(photo_file_path.clone()))
            .data(api::ThumbDirPath::from(thumb_file_path.clone()))
            .data(api::FaceLandmarkPredictorModelFilePath::from(
                face_landmark_predictor_model_file_path.clone(),
            ))
            .data(api::FaceEncoderModelFilePath::from(
                face_encoder_model_file_path.clone(),
            ))
            .service(actix_web::web::scope("/api").configure(api::configure))
            .service(get_photo)
            .service(get_thumb)
            .service(actix_files::Files::new("/static", static_dir_path.clone()))
            .default_service(actix_web::web::to(index))
    })
    .bind((host, port))
    .map_err(|err| ServerError::BindError(err))?
    .run()
    .await
    .map_err(|err| ServerError::RunError(err))
}

#[derive(structopt::StructOpt)]
struct CliOptions {
    #[structopt(long, default_value = "/var/lib/photos/photos.db")]
    db_file_path: std::path::PathBuf,
    #[structopt(long, default_value = "/usr/local/share/photos/www")]
    static_dir_path: std::path::PathBuf,
    #[structopt(long, default_value = "/var/lib/photos/photos")]
    photo_file_path: std::path::PathBuf,
    #[structopt(long, default_value = "/var/lib/photos/thumbs")]
    thumb_file_path: std::path::PathBuf,
    #[structopt(
        long,
        default_value = "/usr/local/share/photos/shape_predictor_68_face_landmarks.dat"
    )]
    face_landmark_predictor_model_file_path: std::path::PathBuf,
    #[structopt(
        long,
        default_value = "/usr/local/share/photos/dlib_face_recognition_resnet_model_v1.dat"
    )]
    face_encoder_model_file_path: std::path::PathBuf,
    #[structopt(short, long, default_value = "0.0.0.0")]
    host: String,
    #[structopt(short, long, default_value = "80")]
    port: u16,
}

#[actix_web::main]
async fn main() {
    use structopt::StructOpt;

    env_logger::init_from_env(env_logger::Env::new().filter("PHOTOSD_LOG"));

    let cli_options = CliOptions::from_args();

    if let Err(error) = run(
        cli_options.db_file_path,
        cli_options.photo_file_path,
        cli_options.thumb_file_path,
        cli_options.static_dir_path,
        cli_options.face_landmark_predictor_model_file_path,
        cli_options.face_encoder_model_file_path,
        &cli_options.host,
        cli_options.port,
    )
    .await
    {
        use std::error::Error;

        println!("Error: {}", error);

        let mut current = error.source();
        if current.is_some() {
            println!("");
            println!("Caused by:");
            while let Some(error) = current {
                println!("  {}", error);
                current = error.source();
            }
        }
    }
}
