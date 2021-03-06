use crate::CowPath;
use dominator::{html, Dom};

type Params = photos_web_core::PhotoQueryParams;
type SharedParams = std::rc::Rc<Params>;

#[derive(Clone)]
pub enum Path {
    Root,
    Photo(i32),
}

impl Path {
    pub fn starts_with(&self, prefix: &Self) -> bool {
        match (self, prefix) {
            (_, Self::Root) => true,
            (Self::Photo(a), Self::Photo(b)) => a == b,
            _ => false,
        }
    }
}

impl<'a> std::convert::TryFrom<std::path::Components<'a>> for Path {
    type Error = crate::FromPathError;

    fn try_from(mut components: std::path::Components<'a>) -> Result<Self, Self::Error> {
        use std::path::Component;
        match components.next() {
            None => Ok(Path::Root),
            Some(Component::Normal(c)) => {
                use std::str::FromStr;
                Ok(Path::Photo(
                    i32::from_str(c.to_string_lossy().as_ref())
                        .map_err(crate::FromPathError::ParseIntError)?,
                ))
            }
            _ => Err(crate::FromPathError::InvalidPath),
        }
    }
}

impl From<Path> for CowPath {
    fn from(path: Path) -> Self {
        match path {
            Path::Root => CowPath::from(""),
            Path::Photo(id) => CowPath::from(format!("{}", id)),
        }
    }
}

pub fn gallery_image(
    state: super::SharedState,
    (id, photo): &(i32, photos_web_core::Photo),
) -> Dom {
    let id = id.clone();
    html!("img", {
        .attribute("src", &format!("/static/thumbs/{}", photo.file_name))
        .attribute("width", &photo.thumb_width.to_string())
        .attribute("height", &photo.thumb_height.to_string())
        .event(move |_: dominator::events::Click| {
            state.path.set(crate::Path::from(Path::Photo(id)))
        })
    })
}

pub fn gallery(state: super::SharedState, params: SharedParams, date: chrono::NaiveDate) -> Dom {
    let update = {
        let state = state.clone();
        let params = params.clone();
        move || {
            let state = state.clone();
            let params = params.clone();
            async move {
                crate::api::get(
                    state.url_with_params(&format!("api/photos/for-day/{}", date), &*params),
                )
                .await
            }
        }
    };

    let render = move |photos: &photos_web_core::Photos| {
        let state = state.clone();
        html!("div", {
            .children(&mut photos.iter().map(move |photo| {
                gallery_image(state.clone(), photo)
            }).collect::<Vec<_>>())
        })
    };

    super::cheese(update, render)
}

pub fn collection_entry(
    state: super::SharedState,
    params: SharedParams,
    date: chrono::NaiveDate,
) -> Dom {
    html!("div", {
        .children(&mut [
            html!("h1", {
                .text(&format!("{}", date.format("%d %B %G")))
            }),
            gallery(state, params, date)
        ])
    })
}

pub fn collection(state: super::SharedState, params: Params) -> Dom {
    let params = SharedParams::new(params);

    let update = {
        let state = state.clone();
        let params = params.clone();
        move || {
            let state = state.clone();
            let params = params.clone();
            async move {
                crate::api::get(state.url_with_params("api/photos/count-per-day", &*params)).await
            }
        }
    };

    let render = move |count_per_day: &Vec<(chrono::NaiveDate, usize)>| {
        html!("div", {
            .children(&mut count_per_day.into_iter().map(|(date, _count)| {
                collection_entry(state.clone(), params.clone(), *date)
            }).collect::<Vec<_>>())
        })
    };

    super::cheese(update, render)
}

pub fn photo(state: super::SharedState, id: i32) -> Dom {
    let render = move |photo: &photos_web_core::Photo| -> Dom {
        html!("img", {
            .attribute("src", &format!("/static/photos/{}", photo.file_name))
            .attribute("width", &photo.image_width.to_string())
            .attribute("height", &photo.image_height.to_string())
        })
    };

    async fn update(
        state: super::SharedState,
        id: i32,
    ) -> Result<photos_web_core::Photo, crate::api::Error> {
        crate::api::get(state.url(&format!("/api/photo/{}", id))).await
    }

    super::cheese(move || update(state.clone(), id), render)
}

pub fn root(state: super::SharedState, sub_path: &Path) -> Dom {
    match sub_path {
        Path::Root => collection(state, Params::default()),
        Path::Photo(id) => photo(state, *id),
    }
}
