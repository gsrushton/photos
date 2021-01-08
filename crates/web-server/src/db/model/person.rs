use crate::db::schema::people;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

#[derive(diesel::AsChangeset, diesel::Queryable)]
#[table_name = "people"]
#[changeset_options(treat_none_as_null = "true")]
pub struct Person {
    pub first_name: String,
    pub middle_names: Option<String>,
    pub surname: String,
    pub display_name: Option<String>,
    pub dob: Option<chrono::NaiveDate>,
}

#[derive(diesel::Insertable)]
#[table_name = "people"]
pub struct NewPerson {
    pub first_name: String,
    pub middle_names: String,
    pub surname: String,
}

impl Person {
    pub async fn insert(db: &crate::db::System) -> Result<i32, crate::db::QueryError> {
        db.run_query({
            let db = db.clone();
            move |db_connection| {
                let _guard = db.people_insertion_guard().lock();

                diesel::insert_into(people::table)
                    .values(&NewPerson {
                        first_name: String::from("Person"),
                        middle_names: String::from("'Photo Bomber'"),
                        surname: String::from("McPerson"),
                    })
                    .execute(&db_connection)?;

                Ok(*people::table
                    .select(people::id)
                    .order(people::id.desc())
                    .limit(1)
                    .load(&db_connection)?
                    .get(0)
                    .unwrap())
            }
        })
        .await
    }

    pub async fn fetch(
        db: &crate::db::System,
        person_id: i32,
    ) -> Result<Option<Self>, crate::db::QueryError> {
        db.run_query(move |db_connection| {
            use crate::db::schema::people::dsl::*;
            people
                .select((first_name, middle_names, surname, display_name, dob))
                .filter(id.eq(person_id))
                .load::<Self>(&db_connection)
        })
        .await
        .map(|mut people| people.pop())
    }

    pub async fn fetch_all(
        db: &crate::db::System,
    ) -> Result<Vec<(i32, Self)>, crate::db::QueryError> {
        db.run_query(move |db_connection| {
            use crate::db::schema::people::dsl::*;
            people
                .select((id, (first_name, middle_names, surname, display_name, dob)))
                .order_by(surname)
                .then_order_by(first_name)
                .then_order_by(id)
                .load::<(i32, Self)>(&db_connection)
        })
        .await
    }

    pub async fn record(
        self,
        db: &crate::db::System,
        person_id: i32,
    ) -> Result<(), crate::db::UpdateQueryError> {
        db.run_query(move |db_connection| {
            use crate::db::schema::people::dsl::*;
            diesel::update(people.filter(id.eq(person_id)))
                .set(self)
                .execute(&db_connection)
        })
        .await
        .map_err(crate::db::UpdateQueryError::QueryError)
        .and_then(|result| match result {
            1 => Ok(()),
            0 => Err(crate::db::UpdateQueryError::NoSuchRecord),
            _ => unreachable!(),
        })
    }

    pub async fn merge(
        db: &crate::db::System,
        dst_id: i32,
        src_id: i32,
    ) -> Result<(), crate::db::UpdateQueryError> {
        db.run_query(move |db_connection| {
            use crate::diesel::Connection;

            db_connection.transaction::<_, diesel::result::Error, _>(|| {
                diesel::delete(crate::db::schema::avatars::table)
                    .filter(crate::db::schema::avatars::person.eq(src_id))
                    .execute(&db_connection)?;

                diesel::update(
                    crate::db::schema::appearances::table
                        .filter(crate::db::schema::appearances::person.eq(src_id)),
                )
                .set(crate::db::schema::appearances::person.eq(dst_id))
                .execute(&db_connection)?;

                diesel::delete(people::table)
                    .filter(people::id.eq(src_id))
                    .execute(&db_connection)
            })
        })
        .await
        .map_err(crate::db::UpdateQueryError::QueryError)
        .and_then(|result| match result {
            1 => Ok(()),
            0 => Err(crate::db::UpdateQueryError::NoSuchRecord),
            _ => unreachable!(),
        })
    }
}

impl From<photos_web_core::Person> for Person {
    fn from(person: photos_web_core::Person) -> Self {
        Self {
            first_name: person.first_name,
            middle_names: person.middle_names,
            surname: person.surname,
            display_name: person.display_name,
            dob: person.dob,
        }
    }
}

impl std::convert::Into<photos_web_core::Person> for Person {
    fn into(self) -> photos_web_core::Person {
        photos_web_core::Person {
            first_name: self.first_name,
            middle_names: self.middle_names,
            surname: self.surname,
            display_name: self.display_name,
            dob: self.dob,
        }
    }
}
