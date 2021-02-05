use dominator::html;

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

pub fn signal<T, E, F, C, U, R>(
    mut container_builder: dominator::DomBuilder<web_sys::HtmlElement>,
    mut u: U,
    mut r: R,
) -> dominator::Dom
where
    U: FnMut() -> F + 'static,
    F: futures::prelude::Future<Output = Result<T, E>>,
    R: FnMut(&T) -> C + 'static,
    C: futures_signals::signal_vec::SignalVec<Item = dominator::Dom> + Unpin + 'static,
    T: 'static,
    E: std::error::Error + 'static,
{
    use futures_signals::{signal::SignalExt, signal_vec::SignalVec};
    use std::rc::Rc;

    struct SingularSignalVec(Option<dominator::Dom>);

    impl SingularSignalVec {
        fn new(dom: dominator::Dom) -> Self {
            Self(Some(dom))
        }
    }

    impl std::marker::Unpin for SingularSignalVec {}

    impl futures_signals::signal_vec::SignalVec for SingularSignalVec {
        type Item = dominator::Dom;

        fn poll_vec_change(
            mut self: std::pin::Pin<&mut Self>,
            _ctx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<futures_signals::signal_vec::VecDiff<Self::Item>>> {
            use futures_signals::signal_vec::VecDiff;
            use std::task::Poll;
            match self.0.take() {
                Some(dom) => Poll::Ready(Some(VecDiff::Push { value: dom })),
                None => Poll::Ready(None),
            }
        }
    }

    fn box_signal_vec<V>(cheese: V) -> Box<dyn SignalVec<Item = dominator::Dom> + Unpin + 'static>
    where
        V: SignalVec<Item = dominator::Dom> + Unpin + 'static,
    {
        Box::new(cheese)
    }

    let result: futures_signals::signal::Mutable<Option<Rc<Result<T, E>>>> =
        futures_signals::signal::Mutable::new(None);

    container_builder =
        container_builder.children_signal_vec(result.signal_cloned().switch_signal_vec(
            move |result| match result {
                Some(result) => match result.as_ref() {
                    Ok(value) => box_signal_vec(r(value)),
                    Err(err) => box_signal_vec(SingularSignalVec::new(error(err))),
                },
                None => box_signal_vec(SingularSignalVec::new(loading())),
            },
        ));

    let update = || {
        wasm_bindgen_futures::spawn_local(async move {
            result.set(Some(Rc::new(u().await)));
        })
    };

    update();

    container_builder.into_dom()
}

pub fn vec<T, E, F, U, R>(
    mut container_builder: dominator::DomBuilder<web_sys::HtmlElement>,
    mut u: U,
    mut r: R,
) -> dominator::Dom
where
    U: FnMut() -> F + 'static,
    F: futures::prelude::Future<Output = Result<T, E>>,
    R: FnMut(&T) -> Vec<dominator::Dom> + 'static,
    T: 'static,
    E: std::error::Error + 'static,
{
    use futures_signals::signal::SignalExt;

    let result = futures_signals::signal::Mutable::new(None);

    container_builder = container_builder.children_signal_vec(
        result
            .signal_ref(move |result| match result {
                Some(Ok(value)) => r(value),
                Some(Err(err)) => vec![error(err)],
                None => vec![loading()],
            })
            .to_signal_vec(),
    );

    let update = || {
        wasm_bindgen_futures::spawn_local(async move {
            result.set(Some(u().await));
        })
    };

    update();

    container_builder.into_dom()
}
