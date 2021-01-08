use crate::db::schema::appearances;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

#[derive(diesel::Insertable)]
#[table_name = "appearances"]
struct NewAppearance {
    pub person: i32,
    pub photo: i32,
    pub reference: bool,
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
    pub face_encoding: crate::db::model::FaceEncoding,
}

#[derive(diesel::Queryable)]
pub struct Appearance {
    pub person: i32,
    pub photo: i32,
    pub reference: bool,
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
    pub face_encoding: crate::db::model::FaceEncoding,
}

impl Into<photos_web_core::Appearance> for Appearance {
    fn into(self) -> photos_web_core::Appearance {
        photos_web_core::Appearance {
            person: self.person,
            photo: self.photo,
            reference: self.reference,
            top: self.top,
            left: self.left,
            bottom: self.bottom,
            right: self.right,
        }
    }
}

impl Appearance {
    pub async fn insert(
        db: &crate::db::System,
        person_id: i32,
        photo_id: i32,
        reference: bool,
        top: i32,
        left: i32,
        bottom: i32,
        right: i32,
        face_encoding: crate::db::model::FaceEncoding,
    ) -> Result<i32, crate::db::QueryError> {
        db.run_query({
            let db = db.clone();
            move |db_connection| {
                let _guard = db.appearances_insertion_guard().lock();

                diesel::insert_into(crate::db::schema::appearances::table)
                    .values(&NewAppearance {
                        person: person_id,
                        photo: photo_id,
                        reference,
                        top,
                        left,
                        bottom,
                        right,
                        face_encoding,
                    })
                    .execute(&db_connection)?;

                Ok(*appearances::table
                    .select(appearances::id)
                    .order(appearances::id.desc())
                    .limit(1)
                    .load(&db_connection)?
                    .get(0)
                    .unwrap())
            }
        })
        .await
    }

    pub async fn fetch_all_for_photo(
        db: &crate::db::System,
        photo_id: i32,
    ) -> Result<Vec<(i32, Appearance)>, crate::db::QueryError> {
        db.run_query(move |db_connection| {
            use crate::db::schema::appearances::dsl::*;
            appearances
                .select((
                    id,
                    (
                        person,
                        photo,
                        reference,
                        top,
                        left,
                        bottom,
                        right,
                        face_encoding,
                    ),
                ))
                .filter(photo.eq(photo_id))
                .order_by(id)
                .load::<(i32, Appearance)>(&db_connection)
        })
        .await
    }
}
