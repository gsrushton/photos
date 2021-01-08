pub mod serde_util;

#[derive(serde::Serialize)]
pub struct ErrorDesc {
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cause: Option<Box<ErrorDesc>>,
}

impl From<&dyn std::error::Error> for ErrorDesc {
    fn from(error: &dyn std::error::Error) -> Self {
        Self {
            description: format!("{}", error),
            cause: error
                .source()
                .map(|source| Box::new(ErrorDesc::from(source))),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Person {
    pub first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_names: Option<String>,
    pub surname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dob: Option<chrono::NaiveDate>,
}

impl Person {
    pub fn display_name(&self) -> std::borrow::Cow<String> {
        Self::make_display_name(&self.first_name, &self.surname, &self.display_name)
    }

    pub fn make_display_name<'a>(
        first_name: &'a str,
        surname: &'a str,
        display_name: &'a Option<String>,
    ) -> std::borrow::Cow<'a, String> {
        use std::borrow::Cow;
        display_name
            .as_ref()
            .map(|display_name| Cow::Borrowed(display_name))
            .unwrap_or(Cow::Owned(format!("{} {}", first_name, surname)))
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct People(#[serde(with = "tuple_vec_map")] Vec<(i32, Person)>);

impl<T, I> From<I> for People
where
    T: Into<Person>,
    I: IntoIterator<Item = (i32, T)>,
{
    fn from(i: I) -> Self {
        Self(
            i.into_iter()
                .map(|(id, person)| (id, person.into()))
                .collect(),
        )
    }
}

impl People {
    pub fn into_inner(self) -> Vec<(i32, Person)> {
        self.0
    }

    pub fn iter(&self) -> impl Iterator<Item = &(i32, Person)> {
        self.0.iter()
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct PhotoQueryParams {
    pub people: Option<Vec<i32>>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Photo {
    pub file_name: String,
    pub image_width: i32,
    pub image_height: i32,
    pub thumb_width: i32,
    pub thumb_height: i32,
    #[serde(with = "crate::serde_util::datetime_ts_seconds_opt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_datetime: Option<chrono::NaiveDateTime>,
    #[serde(with = "chrono::naive::serde::ts_seconds")]
    pub upload_datetime: chrono::NaiveDateTime,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Photos(#[serde(with = "tuple_vec_map")] Vec<(i32, Photo)>);

impl<T, I> From<I> for Photos
where
    T: Into<Photo>,
    I: IntoIterator<Item = (i32, T)>,
{
    fn from(i: I) -> Self {
        Self(
            i.into_iter()
                .map(|(id, photo)| (id, photo.into()))
                .collect(),
        )
    }
}

impl Photos {
    pub fn iter(&self) -> impl Iterator<Item = &(i32, Photo)> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Appearance {
    pub person: i32,
    pub photo: i32,
    pub reference: bool,
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
}

#[derive(serde::Serialize)]
#[serde(transparent)]
pub struct Appearances(#[serde(with = "tuple_vec_map")] Vec<(i32, Appearance)>);

impl<T, I> From<I> for Appearances
where
    T: Into<Appearance>,
    I: IntoIterator<Item = (i32, T)>,
{
    fn from(i: I) -> Self {
        Self(
            i.into_iter()
                .map(|(id, appearance)| (id, appearance.into()))
                .collect(),
        )
    }
}
