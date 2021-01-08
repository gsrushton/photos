use crate::db::schema::avatars;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

#[derive(diesel::Insertable)]
#[table_name = "avatars"]
struct NewAvatar {
    pub person: i32,
    pub appearance: i32,
}

#[derive(diesel::Queryable)]
pub struct Avatar {
    pub file_name: String,
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
}

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("Failed to open image")]
    ImageOpenError(#[source] std::io::Error),
    #[error("Failed to load image")]
    ImageLoadFailed(#[source] crate::image_ext::NewImageExtError),
}

impl Avatar {
    pub async fn insert(
        db: &crate::db::System,
        person_id: i32,
        appearance_id: i32,
    ) -> Result<i32, crate::db::QueryError> {
        db.run_query({
            let db = db.clone();
            move |db_connection| {
                let _guard = db.avatars_insertion_guard().lock();

                diesel::insert_into(avatars::table)
                    .values(&NewAvatar {
                        person: person_id,
                        appearance: appearance_id,
                    })
                    .execute(&db_connection)?;

                Ok(*avatars::table
                    .select(avatars::id)
                    .order(avatars::id.desc())
                    .limit(1)
                    .load(&db_connection)?
                    .get(0)
                    .unwrap())
            }
        })
        .await
    }

    pub async fn fetch_for_person(
        db: &crate::db::System,
        person_id: i32,
    ) -> Result<Option<Self>, crate::db::QueryError> {
        db.run_query(move |db_connection| {
            use crate::db::schema::{appearances, photos};
            avatars::table
                .inner_join(appearances::table.inner_join(photos::table))
                .select((
                    photos::file_name,
                    appearances::top,
                    appearances::left,
                    appearances::bottom,
                    appearances::right,
                ))
                .filter(avatars::person.eq(person_id))
                .load::<Self>(&db_connection)
        })
        .await
        .map(|mut avatars| avatars.pop())
    }

    pub async fn fetch_for_appearance(
        db: &crate::db::System,
        appearance_id: i32,
    ) -> Result<Option<Self>, crate::db::QueryError> {
        db.run_query(move |db_connection| {
            use crate::db::schema::{appearances, photos};
            appearances::table
                .inner_join(photos::table)
                .select((
                    photos::file_name,
                    appearances::top,
                    appearances::left,
                    appearances::bottom,
                    appearances::right,
                ))
                .filter(appearances::id.eq(appearance_id))
                .load::<Self>(&db_connection)
        })
        .await
        .map(|mut avatars| avatars.pop())
    }

    pub fn image(
        &self,
        size: u32,
        photo_dir: &std::path::Path,
    ) -> Result<image::DynamicImage, ImageError> {
        let image = crate::image_ext::ImageExt::new(std::io::BufReader::new(
            std::fs::File::open(photo_dir.join(&self.file_name))
                .map_err(ImageError::ImageOpenError)?,
        ))
        .map_err(ImageError::ImageLoadFailed)?;

        let mut image = image.reorient();

        let centre_x = (self.left + self.right) / 2;
        let centre_y = (self.top + self.bottom) / 2;
        let max_dim = std::cmp::max(self.right - self.left, self.bottom - self.top);

        Ok(image
            .crop(
                std::cmp::max(0i32, centre_x - max_dim) as u32,
                std::cmp::max(0i32, centre_y - max_dim) as u32,
                (max_dim * 2) as u32,
                (max_dim * 2) as u32,
            )
            .resize_exact(size, size, image::imageops::FilterType::Lanczos3))
    }
}
