use crate::CowPath;
use dominator::{clone, html, Dom};

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
        photos
            .iter()
            .map(move |photo| gallery_image(state.clone(), photo))
            .collect::<Vec<_>>()
    };

    super::def::vec(
        dominator::DomBuilder::new_html("ul").class("photo-gallery"),
        update,
        render,
    )
}

pub fn collection_entry(
    state: super::SharedState,
    params: SharedParams,
    date: chrono::NaiveDate,
) -> Dom {
    html!("li", {
        .class("photo-collection-entry")
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
        count_per_day
            .into_iter()
            .map(|(date, _count)| collection_entry(state.clone(), params.clone(), *date))
            .collect::<Vec<_>>()
    };

    super::def::vec(
        dominator::DomBuilder::new_html("ul").class("photo-collection"),
        update,
        render,
    )
}

type MutableAppearances =
    futures_signals::signal_vec::MutableVec<(i32, photos_web_core::Appearance)>;

fn frame(photo: &photos_web_core::Photo) -> Dom {
    html!("div", {
        .class("frame")
        .children(&mut [
            html!("div", {
                .class("mount")
                .children(&mut [
                    html!("img", {
                        .attribute("src", &format!("/static/photos/{}", photo.file_name))
                    })
                ])
            })
        ])
    })
}

#[derive(Clone)]
enum InfoPanels {
    AppearanceGallery,
    AppearanceDetails(i32, photos_web_core::Appearance),
}

fn info(state: super::SharedState, photo_id: i32) -> Dom {
    use futures_signals::{signal::SignalExt, signal_vec::SignalVecExt};

    let selected_appearance = std::sync::Arc::new(futures_signals::signal::Mutable::new(None));

    let panels = futures_signals::signal_vec::MutableVec::new_with_values(vec![
        InfoPanels::AppearanceGallery,
    ]);

    let panel_signal_vec = panels.signal_vec_cloned();

    html!("div", {
        .class("info")
        .future(selected_appearance.signal_cloned().for_each(move |appearance| {
            if panels.lock_ref().len() > 1 {
                panels.lock_mut().pop();
            }
            if let Some((id, appearance)) = appearance {
                panels
                    .lock_mut()
                    .push_cloned(InfoPanels::AppearanceDetails(id, appearance))
            }
            futures::future::ready(())
        }))
        .children_signal_vec(
            panel_signal_vec.map(move |panel| match panel {
                InfoPanels::AppearanceGallery => {
                    appearance_gallery(state.clone(), photo_id, selected_appearance.clone())
                },
                InfoPanels::AppearanceDetails(id, appearance) => {
                    appearance_detail(state.clone(), id, appearance)
                }
            })
        )
    })
}

fn appearance_detail(
    state: crate::SharedState,
    appearance_id: i32,
    appearance: photos_web_core::Appearance,
) -> Dom {
    let render = move |person: &photos_web_core::Person| -> Vec<Dom> {
        vec![html!("div", {
            .text(&format!("{}", person.display_name()))
        })]
    };

    async fn update(
        state: crate::SharedState,
        person_id: i32,
    ) -> Result<photos_web_core::Person, crate::api::Error> {
        crate::api::get(state.url(&format!("/api/people/{}", person_id))).await
    }

    crate::def::vec(
        dominator::DomBuilder::new_html("div"),
        move || update(state.clone(), appearance.person),
        render,
    )
}

fn appearance_gallery(
    state: super::SharedState,
    photo_id: i32,
    selected_appearance: std::sync::Arc<
        futures_signals::signal::Mutable<Option<(i32, photos_web_core::Appearance)>>,
    >,
) -> Dom {
    let render = move |appearances: &std::sync::Arc<MutableAppearances>| {
        use futures_signals::{signal::SignalExt, signal_vec::SignalVecExt};
        let selected_appearance = selected_appearance.clone();
        appearances
            .signal_vec_cloned()
            .map(move |(id, appearance)| {
                html!("img", {
                    .class("avatar")
                    .class_signal("selected", selected_appearance.signal_cloned().map(
                        move |appearance| match appearance {
                            Some((selected_id, _)) => selected_id == id,
                            None => false
                        }
                    ))
                    .attribute("src", &format!("/api/people/{}/avatar?size=64", appearance.person))
                    .event({
                        let selected_appearance = selected_appearance.clone();
                        move |_: dominator::events::Click| {
                            selected_appearance.set(Some((id, appearance.clone())))
                        }
                    })
                })
            })
    };

    async fn update(
        state: super::SharedState,
        id: i32,
    ) -> Result<std::sync::Arc<MutableAppearances>, crate::api::Error> {
        let appearances: photos_web_core::Appearances =
            crate::api::get(state.url(&format!("/api/photos/{}/appearances", id))).await?;

        let appearances = std::sync::Arc::new(MutableAppearances::new_with_values(
            appearances.into_inner(),
        ));

        Ok(appearances)
    }

    crate::def::signal(
        dominator::DomBuilder::new_html("div").class("appearance-gallery"),
        move || update(state.clone(), photo_id),
        render,
    )
}

pub fn photo(state: super::SharedState, id: i32) -> Dom {
    fn render(state: super::SharedState, id: i32, photo: &photos_web_core::Photo) -> Vec<Dom> {
        vec![frame(photo), info(state, id)]
    }

    async fn update(
        state: super::SharedState,
        id: i32,
    ) -> Result<photos_web_core::Photo, crate::api::Error> {
        crate::api::get(state.url(&format!("/api/photo/{}", id))).await
    }

    super::def::vec(
        dominator::DomBuilder::new_html("div").class("photo"),
        {
            let state = state.clone();
            move || update(state.clone(), id)
        },
        move |photo| render(state.clone(), id, photo),
    )
}

pub fn root(state: super::SharedState, sub_path: &Path) -> Dom {
    match sub_path {
        Path::Root => collection(state, Params::default()),
        Path::Photo(id) => photo(state, *id),
    }
}
