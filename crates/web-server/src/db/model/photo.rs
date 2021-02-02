use crate::db::schema::{appearances, photos};
use diesel::{
    sql_types::{Nullable, Timestamp},
    ExpressionMethods, QueryDsl, RunQueryDsl,
};

sql_function!(fn ifnull(x: Nullable<Timestamp>, y: Timestamp) -> Timestamp);

#[derive(diesel::Insertable)]
#[table_name = "photos"]
struct NewPhoto {
    pub digest: crate::db::model::Digest,
    pub file_name: String,
    pub image_width: i32,
    pub image_height: i32,
    pub thumb_width: i32,
    pub thumb_height: i32,
    pub original_datetime: Option<chrono::NaiveDateTime>,
    pub upload_datetime: chrono::NaiveDateTime,
}

#[derive(diesel::Queryable)]
pub struct Photo {
    pub digest: crate::db::model::Digest,
    pub file_name: String,
    pub image_width: i32,
    pub image_height: i32,
    pub thumb_width: i32,
    pub thumb_height: i32,
    pub original_datetime: Option<chrono::NaiveDateTime>,
    pub upload_datetime: chrono::NaiveDateTime,
}

impl Photo {
    pub async fn insert(
        db: &crate::db::System,
        digest: crate::db::model::Digest,
        file_name: String,
        image_width: u32,
        image_height: u32,
        thumb_width: u32,
        thumb_height: u32,
        original_datetime: Option<chrono::NaiveDateTime>,
        upload_datetime: chrono::NaiveDateTime,
    ) -> Result<i32, crate::db::QueryError> {
        db.run_query({
            let db = db.clone();
            move |db_connection| {
                let _guard = db.photos_insertion_guard().lock();

                diesel::insert_into(crate::db::schema::photos::table)
                    .values(&NewPhoto {
                        digest,
                        file_name,
                        image_width: image_width as i32,
                        image_height: image_height as i32,
                        thumb_width: thumb_width as i32,
                        thumb_height: thumb_height as i32,
                        original_datetime,
                        upload_datetime,
                    })
                    .execute(&db_connection)?;

                Ok(*photos::table
                    .select(photos::id)
                    .order(photos::id.desc())
                    .limit(1)
                    .load::<i32>(&db_connection)?
                    .get(0)
                    .unwrap())
            }
        })
        .await
    }

    pub async fn count_per_day(
        db: &crate::db::System,
        people: Vec<i32>,
    ) -> Result<Vec<(chrono::NaiveDate, usize)>, crate::db::QueryError> {
        db.run_query(|db_connection| {
            let datetime = ifnull(photos::original_datetime, photos::upload_datetime);
            people
                .into_iter()
                .fold(
                    photos::table
                        .left_join(appearances::table)
                        .select(datetime)
                        .distinct()
                        .order(datetime.desc())
                        .into_boxed(),
                    |query, person| query.filter(appearances::person.eq(person)),
                )
                .load::<chrono::NaiveDateTime>(&db_connection)
        })
        .await
        .map(|datetimes| {
            use itertools::Itertools;
            datetimes
                .into_iter()
                .map(|datetime| datetime.date())
                .dedup_with_count()
                .map(|(count, date)| (date, count))
                .collect()
        })
    }

    pub async fn fetch_all_for_day(
        db: &crate::db::System,
        date: chrono::NaiveDate,
        people: Vec<i32>,
    ) -> Result<Vec<(i32, Self)>, crate::db::QueryError> {
        db.run_query(move |db_connection| {
            let datetime = ifnull(photos::original_datetime, photos::upload_datetime);

            people
                .into_iter()
                .fold(
                    photos::table
                        .left_join(appearances::table)
                        .select((
                            photos::id,
                            (
                                photos::digest,
                                photos::file_name,
                                photos::image_width,
                                photos::image_height,
                                photos::thumb_width,
                                photos::thumb_height,
                                photos::original_datetime,
                                photos::upload_datetime,
                            ),
                        ))
                        .distinct()
                        .filter(datetime.ge(date.and_hms(0, 0, 0)))
                        .filter(datetime.lt(date.succ().and_hms(0, 0, 0)))
                        .order_by(datetime)
                        .then_order_by(photos::id)
                        .into_boxed(),
                    |query, person| query.filter(appearances::person.eq(person)),
                )
                .load::<(i32, Self)>(&db_connection)
        })
        .await
    }
}

impl std::convert::Into<photos_web_core::Photo> for Photo {
    fn into(self) -> photos_web_core::Photo {
        photos_web_core::Photo {
            file_name: self.file_name,
            image_width: self.image_width,
            image_height: self.image_height,
            thumb_width: self.thumb_width,
            thumb_height: self.thumb_height,
            original_datetime: self.original_datetime,
            upload_datetime: self.upload_datetime,
        }
    }
}
