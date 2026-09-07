use std::rc::Rc;

use retgui::elements::{Container, Element, State, Window};
use retgui::style::{Display, FlexDirection};
use retgui::{App, States, pct};

use crate::WebsiteGlobalState;
use crate::docs::docs;
use crate::examples::examples;
use crate::index::index_page;
use crate::navbar::navbar;
use crate::theme::BODY_BACKGROUND_COLOR;

pub type NavigateFn = Rc<dyn Fn(&str, &mut App, &mut States) + 'static>;

pub struct Router {
    state: State<RouterState>,
}

struct RouterState {
    root: Option<Window>,
    global_state: State<WebsiteGlobalState>,
    index: Option<Container>,
    docs: Option<Container>,
    examples: Option<Container>,
}

impl Router {
    pub fn new(app: &mut App, states: &mut States, global_state: State<WebsiteGlobalState>) -> Self {
        let state = states.insert(RouterState {
            root: None,
            global_state,
            index: None,
            docs: None,
            examples: None,
        });
        let navigate: NavigateFn = Rc::new(move |route, app, states| {
            navigate_to(state, app, states, route);
        });

        let navigation = navbar(app, navigate.clone());
        let root = Window::new(app, "RetGui GUI");
        root.set_display(app, Display::Flex);
        root.set_flex_direction(app, FlexDirection::Column);
        root.set_width(app, pct(100));
        root.set_height(app, pct(100));
        root.set_background_color(app, BODY_BACKGROUND_COLOR);
        root.push(app, navigation);
        let index = index_page(app, navigate.clone());
        let docs = docs(app, navigate.clone());
        let examples = examples(app, states, global_state, navigate);

        let router = state.borrow_mut(states);
        router.root = Some(root);
        router.index = Some(index);
        router.docs = Some(docs);
        router.examples = Some(examples);
        Self { state }
    }

    pub fn navigate(&self, app: &mut App, states: &mut States) {
        let global_state = self.state.borrow(states).global_state;
        let route = global_state.borrow(states).get_route();
        navigate_to(self.state, app, states, &route);
    }
}

fn navigate_to(state: State<RouterState>, app: &mut App, states: &mut States, route: &str) {
    let (global_state, root, page) = {
        let router = state.borrow(states);
        let base = route.split('/').find(|part| !part.is_empty()).unwrap_or("");
        let page = match base {
            "docs" => router.docs.expect("docs page was not initialized"),
            "examples" => router.examples.expect("examples page was not initialized"),
            _ => router.index.expect("index page was not initialized"),
        };
        let root = router.root.expect("router root was not initialized");
        (router.global_state, root, page)
    };

    global_state.borrow_mut(states).set_route(route);
    if let Some(current) = root.children(app).get(1).copied() {
        root.remove_child(app, current).expect("failed to remove routed page");
    }
    root.push(app, page);
}
