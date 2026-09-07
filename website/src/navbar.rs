use retgui::elements::{Container, Element, Text};
use retgui::style::{AlignItems, Display, FontWeight, JustifyContent, Unit};
use retgui::{App, pct, px, rgb};

use crate::link::Link;
use crate::router::NavigateFn;
use crate::theme::{NAVBAR_BACKGROUND_COLOR, NAVBAR_TEXT_COLOR, wrapper};

pub const NAVBAR_HEIGHT: f32 = 60.0;

fn create_link(app: &mut App, navigate: NavigateFn, label: &str, route: &str) -> Container {
    let route_owned = route.to_string();
    let text = Text::new(app, label);
    text.set_id(app, &format!("route_{route}"));
    text.set_margin(app, px(0), px(12), px(0), px(0));
    text.set_font_size(app, 16.0);
    text.set_selectable(app, false);
    text.set_color(app, NAVBAR_TEXT_COLOR);
    let link = Link(app, move |app| navigate(&route_owned, app));
    link.push(app, text);
    link
}

pub fn navbar(app: &mut App, navigate: NavigateFn) -> Container {
    let brand = create_link(app, navigate.clone(), "RetGui", "/");
    brand.set_font_size(app, 32.0);
    brand.set_font_weight(app, FontWeight::BOLD);
    brand.set_margin(app, px(0), px(24), px(0), px(0));
    let home = create_link(app, navigate.clone(), "Home", "/");
    let docs = create_link(app, navigate.clone(), "Docs", "/docs");
    let examples = create_link(app, navigate, "Examples", "/examples");
    let links = Container::new(app);
    links.set_display(app, Display::Flex);
    links.set_justify_content(app, JustifyContent::Center);
    links.set_align_items(app, AlignItems::Center);
    links.push(app, brand);
    links.push(app, home);
    links.push(app, docs);
    links.push(app, examples);
    let inner = wrapper(app);
    inner.set_display(app, Display::Flex);
    inner.set_justify_content(app, JustifyContent::SpaceBetween);
    inner.set_align_items(app, AlignItems::Center);
    inner.push(app, links);
    let border = rgb(240, 240, 240);
    let container = Container::new(app);
    container.set_width(app, pct(100));
    container.set_height(app, Unit::Px(NAVBAR_HEIGHT));
    container.set_min_height(app, Unit::Px(NAVBAR_HEIGHT));
    container.set_max_height(app, Unit::Px(NAVBAR_HEIGHT));
    container.set_border_width(app, px(0), px(0), px(2), px(0));
    container.set_border_color(app, border, border, border, border);
    container.set_background_color(app, NAVBAR_BACKGROUND_COLOR);
    container.push(app, inner);
    container
}
