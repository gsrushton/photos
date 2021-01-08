use diesel::sql_types::{Nullable, Timestamp};

pub mod appearance;
pub mod avatar;
pub mod digest;
pub mod face_encoding;
pub mod person;
pub mod photo;

pub use appearance::Appearance;
pub use avatar::Avatar;
pub use digest::Digest;
pub use face_encoding::FaceEncoding;
pub use person::Person;
pub use photo::Photo;

sql_function!(fn coalesc_date(x: Nullable<Timestamp>, y: Timestamp) -> Timestamp);
