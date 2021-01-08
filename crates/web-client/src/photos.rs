use dominator::{html, Dom};

type Params = photos_web_core::PhotoQueryParams;
type SharedParams = std::rc::Rc<Params>;

pub fn gallery_image((_id, photo): &(i32, photos_web_core::Photo)) -> Dom {
    html!("img", {
        .attribute("src", &format!("/static/thumbs/{}", photo.file_name))
        .attribute("width", &photo.thumb_width.to_string())
        .attribute("height", &photo.thumb_height.to_string())
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
        html!("div", {
            .children(&mut photos.iter().map(gallery_image).collect::<Vec<_>>())
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

pub fn root(state: super::SharedState) -> Dom {
    collection(state, Params::default())
}
