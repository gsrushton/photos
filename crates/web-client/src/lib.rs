use dominator::{clone, html};
use futures_signals::signal::SignalExt;

mod api;
mod cow_path;
mod net;
mod people;
mod photos;

use cow_path::CowPath;

fn cheese<T, E, F, U, R>(mut u: U, mut r: R) -> dominator::Dom
where
    U: FnMut() -> F + 'static,
    R: FnMut(&T) -> dominator::Dom + 'static,
    F: futures::prelude::Future<Output = Result<T, E>>,
    T: 'static,
    E: std::error::Error + 'static,
{
    let result = futures_signals::signal::Mutable::new(None);

    fn loading() -> dominator::Dom {
        html!("span", {
            .text("loading")
        })
    }

    fn error(err: &(dyn std::error::Error + 'static)) -> dominator::Dom {
        html!("div", {
            .text(&format!("{}", err))
            .children(&mut err.source().iter().map(|err| error(*err)).collect::<Vec<_>>())
        })
    }

    let cake = html!("div", {
        .child_signal(result.signal_ref(move |result| {
            Some(match result {
                None => loading(),
                Some(Ok(value)) => r(value),
                Some(Err(err)) => error(err),
            })
        }))
    });

    let update = || {
        wasm_bindgen_futures::spawn_local(async move {
            result.set(Some(u().await));
        })
    };

    update();

    cake
}

pub struct State {
    origin: url::Url,
    path: futures_signals::signal::Mutable<Path>,
}

impl State {
    pub fn new(origin: url::Url, path: Path) -> Self {
        Self {
            origin,
            path: futures_signals::signal::Mutable::new(path),
        }
    }

    pub fn url(&self, path: &str) -> url::Url {
        self.origin.join(path).unwrap()
    }

    pub fn url_with_params<P>(&self, path: &str, params: &P) -> url::Url
    where
        P: serde::Serialize,
    {
        let mut url = self.url(path);
        url.set_query(Some(&serde_qs::to_string(params).unwrap()));
        url
    }
}

type SharedState = std::rc::Rc<State>;

#[derive(Clone)]
pub enum Path {
    Photos(photos::Path),
    People(people::Path),
    NotFound(std::path::PathBuf),
}

impl Path {
    pub fn try_from_path(path: &std::path::Path) -> Result<Self, FromPathError> {
        use std::convert::TryFrom;
        use std::path::Component;
        let mut components = path.components();
        match components.next() {
            Some(Component::RootDir) => Ok(Self::try_from(components)?),
            _ => Err(FromPathError::InvalidPath),
        }
    }

    pub fn starts_with(&self, prefix: &Self) -> bool {
        match (self, prefix) {
            (Self::Photos(a), Self::Photos(b)) => a.starts_with(b),
            (Self::People(a), Self::People(b)) => a.starts_with(b),
            _ => false,
        }
    }
}

impl From<photos::Path> for Path {
    fn from(sub_path: photos::Path) -> Self {
        Self::Photos(sub_path)
    }
}

impl From<people::Path> for Path {
    fn from(sub_path: people::Path) -> Self {
        Self::People(sub_path)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FromPathError {
    #[error("Invalid path")]
    InvalidPath,
    #[error("Failed to parse path component")]
    ParseIntError(#[source] std::num::ParseIntError),
}

impl<'a> std::convert::TryFrom<std::path::Components<'a>> for Path {
    type Error = FromPathError;

    fn try_from(mut components: std::path::Components<'a>) -> Result<Self, Self::Error> {
        use std::path::Component;
        match components.next() {
            None => Ok(Path::Photos(photos::Path::Root)),
            Some(Component::Normal(c)) if c == "photos" => {
                Ok(Path::from(photos::Path::try_from(components)?))
            }
            Some(Component::Normal(c)) if c == "people" => {
                Ok(Path::from(people::Path::try_from(components)?))
            }
            _ => Err(FromPathError::InvalidPath),
        }
    }
}

impl From<Path> for CowPath {
    fn from(path: Path) -> Self {
        match path {
            Path::Photos(sub_path) => CowPath::from("photos").join(CowPath::from(sub_path)),
            Path::People(sub_path) => CowPath::from("people").join(CowPath::from(sub_path)),
            Path::NotFound(path) => CowPath::from(path),
        }
    }
}

fn nav_bar(state: &State) -> dominator::Dom {
    let path = &state.path;

    let make_link = move |name, path_prefix: Path| {
        html!("li", {
            .text(name)
            .class_signal("active", path.signal_ref(clone!(path_prefix => move |path| {
                path.starts_with(&path_prefix)
            })))
            .event(clone!(path => move |_: dominator::events::Click| {
                path.set(path_prefix.clone())
            }))
        })
    };

    html!("ul", {
        .attribute("id", "nav-bar")
        .children(&mut [
            make_link("Photos", Path::Photos(photos::Path::Root)),
            make_link("People", Path::People(people::Path::Root))
        ])
    })
}

fn path_not_found() -> dominator::Dom {
    html!("span", {
        .text("Unrecognised path")
    })
}

fn root(state: SharedState) -> dominator::Dom {
    html!("div", {
        .attribute("id", "root")
        .children_signal_vec(state.path.signal_cloned().map(move |path| {
            vec![
                nav_bar(state.as_ref()),
                match path {
                    Path::Photos(sub_path) => photos::root(state.clone(), &sub_path),
                    Path::People(sub_path) => people::root(state.clone(), &sub_path),
                    Path::NotFound(_) => path_not_found(),
                },
            ]
        }).to_signal_vec())
    })
}

pub struct EventListenerHandle<E>
where
    E: std::convert::AsRef<web_sys::EventTarget>,
{
    target: E,
    event: String,
    closure: wasm_bindgen::closure::Closure<dyn FnMut()>,
}

impl<E> Drop for EventListenerHandle<E>
where
    E: std::convert::AsRef<web_sys::EventTarget>,
{
    fn drop(&mut self) {
        use wasm_bindgen::JsCast;
        if let Err(_) = self
            .target
            .as_ref()
            .remove_event_listener_with_callback(&self.event, self.closure.as_ref().unchecked_ref())
        {
            log::error!("Failed to remove event listener");
        }
    }
}

pub fn add_event_listener<E, F>(
    target: E,
    event: String,
    f: F,
) -> Result<EventListenerHandle<E>, ()>
where
    E: std::convert::AsRef<web_sys::EventTarget>,
    F: FnMut() + 'static,
{
    use wasm_bindgen::JsCast;

    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(f) as Box<dyn FnMut()>);

    target
        .as_ref()
        .add_event_listener_with_callback(&event, closure.as_ref().unchecked_ref())
        .map_err(|err| ())?;

    Ok(EventListenerHandle {
        target,
        event,
        closure,
    })
}

pub fn crackers() -> Path {
    let window = web_sys::window().unwrap();

    Path::try_from_path(std::path::Path::new(&window.location().pathname().unwrap())).unwrap()
}

pub fn cake(state: SharedState) {
    let window = web_sys::window().unwrap();

    std::mem::forget(
        add_event_listener(window.clone(), String::from("popstate"), {
            // TODO How not to call push_state?
            // let state = state.clone();
            move || {} //state.path.set(crackers())
        })
        .unwrap(),
    );

    let history = window.history().unwrap();

    wasm_bindgen_futures::spawn_local(state.path.signal_cloned().for_each(move |path| {
        let url = state
            .origin
            .join(CowPath::from(path).as_ref().to_str().unwrap())
            .unwrap();

        history
            .push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(url.as_ref()))
            .unwrap();

        futures::future::ready(())
    }));
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    wasm_logger::init(wasm_logger::Config::default());

    let window = web_sys::window().unwrap();

    let state = std::rc::Rc::new(State::new(
        url::Url::parse(&window.location().origin().unwrap()).unwrap(),
        crackers(),
    ));

    cake(state.clone());

    dominator::append_dom(&dominator::body(), root(state));
}
