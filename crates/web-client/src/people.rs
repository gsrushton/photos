use crate::CowPath;
use dominator::{html, Dom};

#[derive(Clone)]
pub enum Path {
    Root,
    Person(i32),
}

impl Path {
    pub fn starts_with(&self, prefix: &Self) -> bool {
        match (self, prefix) {
            (_, Self::Root) => true,
            (Self::Person(a), Self::Person(b)) => a == b,
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
                Ok(Path::Person(
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
            Path::Person(id) => CowPath::from(format!("{}", id)),
        }
    }
}

pub fn avatar(id: i32) -> Dom {
    html!("img", {
        .class("avatar")
        .attribute("src", &format!("/api/people/{}/avatar", id))
    })
}

fn plate(
    state: crate::SharedState,
    id: i32,
    person: &photos_web_core::Person,
    merge_tx: futures::channel::mpsc::Sender<(i32, i32)>,
) -> Dom {
    let get_dragged_id = move |data_transfer: &web_sys::DataTransfer| -> Option<i32> {
        match data_transfer.get_data("application/person") {
            Ok(id_str) if id_str.len() > 0 => id_str.parse().ok().and_then(|foreign_id| {
                if foreign_id == id {
                    None
                } else {
                    Some(foreign_id)
                }
            }),
            _ => None,
        }
    };

    html!("li", {
        .attribute("draggable", "true")
        .children(&mut [
            avatar(id),
            html!("span", {
                .text(&format!("{}", person.display_name()))
            })
        ])
        .event(move |_: dominator::events::Click| {
            state.path.set(Path::Person(id).into())
        })
        .event(move |event: dominator::events::DragStart| {
            let data_transfer = event.data_transfer().unwrap();
            data_transfer.set_data("application/person", &id.to_string()).unwrap();
            data_transfer.set_effect_allowed("move");
        })
        .event_preventable(move |event: dominator::events::DragEnter| {
            let data_transfer = event.data_transfer().unwrap();
            if let Some(_) = get_dragged_id(&data_transfer) {
                event.prevent_default();
                data_transfer.set_drop_effect("move");
            }
        })
        .event_preventable(move |event: dominator::events::DragOver| {
            let data_transfer = event.data_transfer().unwrap();
            if let Some(_) = get_dragged_id(&data_transfer) {
                event.prevent_default();
                data_transfer.set_drop_effect("move");
            }
        })
        .event_preventable(move |event: dominator::events::Drop| {
            use futures::SinkExt;
            let data_transfer = event.data_transfer().unwrap();
            if let Some(foreign_id) = get_dragged_id(&data_transfer) {
                event.prevent_default();
                wasm_bindgen_futures::spawn_local({
                    let mut merge_tx = merge_tx.clone();
                    async move {
                        let _ = merge_tx.send((foreign_id, id)).await;
                    }
                })
            }
        })
    })
}

fn people(state: crate::SharedState) -> Dom {
    type MutablePeople = futures_signals::signal_vec::MutableVec<(i32, photos_web_core::Person)>;

    let render = {
        let state = state.clone();
        move |(people, merge_tx): &(
            std::sync::Arc<MutablePeople>,
            futures::channel::mpsc::Sender<(i32, i32)>,
        )| {
            use futures_signals::signal_vec::SignalVecExt;
            let state = state.clone();
            let merge_tx = merge_tx.clone();
            html!("ul", {
                .attribute("id", "people")
                .children_signal_vec(people.signal_vec_cloned().map(move |(id, person)| {
                    plate(state.clone(), id, &person, merge_tx.clone())
                }))
            })
        }
    };

    async fn update(
        state: crate::SharedState,
    ) -> Result<
        (
            std::sync::Arc<MutablePeople>,
            futures::channel::mpsc::Sender<(i32, i32)>,
        ),
        crate::api::Error,
    > {
        let people: photos_web_core::People = crate::api::get(state.url("/api/people")).await?;

        let people = std::sync::Arc::new(MutablePeople::new_with_values(people.into_inner()));

        let (merge_tx, mut merge_rx) = futures::channel::mpsc::channel(2);
        wasm_bindgen_futures::spawn_local({
            let people = people.clone();
            async move {
                use futures::StreamExt;
                while let Some((src, dst)) = merge_rx.next().await {
                    match crate::api::post(
                        state.url(&format!("/api/people/{}/merge/{}", dst, src)),
                        (),
                    )
                    .await
                    {
                        Ok(()) => {
                            let mut people = people.lock_mut();
                            if let Some(index) = people.iter().position(|(id, _)| *id == src) {
                                people.remove(index);
                            }
                        }
                        Err(err) => {
                            // Report error to user
                        }
                    }
                }
            }
        });

        Ok((people, merge_tx))
    };

    crate::cheese(move || update(state.clone()), render)
}

fn person(state: crate::SharedState, id: i32) -> Dom {
    use futures_signals::signal::Mutable;

    fn input<E, U>(person: Mutable<photos_web_core::Person>, class: &str, e: E, mut u: U) -> Dom
    where
        E: FnMut(&photos_web_core::Person) -> String + 'static,
        U: FnMut(&mut photos_web_core::Person, String) + 'static,
    {
        html!("input", {
            .class(class)
            .property_signal("value", person.signal_ref(e))
            .event(move |event: dominator::events::Input| {
                u(&mut *person.lock_mut(), event.value().unwrap_or_else(|| "".into()))
            })
        })
    }

    fn labelled_input<E, U>(
        label: &str,
        person: Mutable<photos_web_core::Person>,
        e: E,
        u: U,
    ) -> Dom
    where
        E: FnMut(&photos_web_core::Person) -> String + 'static,
        U: FnMut(&mut photos_web_core::Person, String) + 'static,
    {
        html!("label", {
            .children(&mut [
                html!("span", {
                    .text(label)
                }),
                input(person, "standard", e, u)
            ])
        })
    }

    let render = {
        let state = state.clone();
        move |(id, person): &(i32, Mutable<photos_web_core::Person>)| {
            let name_fields = html!("div", {
                .class("field-row")
                .children(&mut [
                    labelled_input(
                        "First Name",
                        person.clone(),
                        |person| person.first_name.clone(),
                        |person, value| person.first_name = value,
                    ),
                    labelled_input(
                        "Middle Names",
                        person.clone(),
                        |person| person.middle_names.clone().unwrap_or_else(|| String::from("")),
                        |person, value| person.middle_names = if value.len() > 0 {
                            Some(value)
                        } else {
                            None
                        },
                    ),
                    labelled_input(
                        "Surname",
                        person.clone(),
                        |person| person.surname.clone(),
                        |person, value| person.surname = value,
                    ),
                ])
            });

            let form = html!("div", {
                .class("form")
                .children(&mut [
                    input(
                        person.clone(),
                        "title",
                        |person| person.display_name().into_owned(),
                        |person, value| person.display_name = if value.len() > 0 {
                            Some(value)
                        } else {
                            None
                        }
                    ),
                    name_fields
                ])
            });

            let header = html!("div", {
                .attribute("id", "header")
                .children(&mut [
                    avatar(*id),
                    form
                ])
            });

            html!("div", {
                .attribute("id", "person")
                .children(&mut [
                    header,
                    crate::photos::collection(state.clone(), photos_web_core::PhotoQueryParams {
                        people: Some(vec![*id])
                    })
                ])
            })
        }
    };

    let update = move || {
        let state = state.clone();
        async move {
            crate::api::get(state.origin.join(&format!("/api/people/{}", id)).unwrap())
                .await
                .map(|person: photos_web_core::Person| {
                    use futures_signals::signal::SignalExt;

                    let person = Mutable::new(person);

                    let state = state.clone();
                    wasm_bindgen_futures::spawn_local(person.signal_cloned().for_each(
                        move |person| {
                            let state = state.clone();
                            async move {
                                match crate::api::put(
                                    state.origin.join(&format!("/api/people/{}", id)).unwrap(),
                                    person,
                                )
                                .await
                                {
                                    Ok(()) => {}
                                    Err(_) => {
                                        // TODO report the error to the user
                                    }
                                }
                            }
                        },
                    ));

                    (id, person)
                })
        }
    };

    crate::cheese(update, render)
}

pub fn root(state: crate::SharedState, sub_path: &Path) -> Dom {
    match sub_path {
        Path::Root => people(state),
        Path::Person(id) => person(state, *id),
    }
}
