use retgui::elements::{Container, Element, Window};
use retgui::style::{Display, FlexDirection};
use retgui::{App, pct};

use crate::WebsiteState;
use crate::docs::docs;
use crate::examples::examples;
use crate::index::index_page;
use crate::navbar::navbar;
use crate::theme::BODY_BACKGROUND_COLOR;

pub type NavigateFn = fn(&str, &mut App<WebsiteState>, &mut WebsiteState);

pub struct Router {
    root: Window,
    index: Container,
    docs: Container,
    examples: Container,
}

impl Router {
    pub fn new(app: &mut App<WebsiteState>, route: &str) -> (Self, usize) {
        let navigation = navbar(app, navigate_to);
        let root = Window::new(app, "RetGui GUI");
        root.set_display(app, Display::Flex);
        root.set_flex_direction(app, FlexDirection::Column);
        root.set_width(app, pct(100));
        root.set_height(app, pct(100));
        root.set_background_color(app, BODY_BACKGROUND_COLOR);
        root.push(app, navigation);
        let index = index_page(app, navigate_to);
        let docs = docs(app, navigate_to);
        let (examples, selected_example) = examples(app, route, navigate_to);

        (
            Self {
                root,
                index,
                docs,
                examples,
            },
            selected_example,
        )
    }

    fn navigate(&self, app: &mut App<WebsiteState>, route: &str) {
        let base = route.split('/').find(|part| !part.is_empty()).unwrap_or("");
        let page = match base {
            "docs" => self.docs,
            "examples" => self.examples,
            _ => self.index,
        };
        if let Some(current) = self.root.children(app).get(1).copied() {
            self.root
                .remove_child(app, current)
                .expect("failed to remove routed page");
        }
        self.root.push(app, page);
    }
}

pub fn navigate_to(route: &str, app: &mut App<WebsiteState>, state: &mut WebsiteState) {
    state.global.set_route(route);
    state.router.navigate(app, route);
}
