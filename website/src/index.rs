use retgui::elements::{Container, Element, Text};
use retgui::style::{Display, FlexDirection, FlexWrap, FontWeight, Overflow, Unit};
use retgui::{App, Color, palette, pct, px, rgb};

use crate::link::Link;
use crate::router::NavigateFn;
use crate::theme::{WRAPPER_PADDING_LEFT, WRAPPER_PADDING_RIGHT, wrapper};
use crate::web_link::WebLink;

fn hero_intro(app: &mut App, navigate: NavigateFn) -> Container {
    let heading = Text::new(app, "A Reactive GUI Framework for Rust");
    heading.set_color(app, Color::WHITE);
    heading.set_font_size(app, 56.0);
    heading.set_line_height(app, 1.0);
    heading.set_max_width(app, px(680));
    heading.set_font_weight(app, FontWeight::BOLD);
    let subtitle = Text::new(app, "Build your UI with regular Rust code.");
    subtitle.set_line_height(app, 1.0);
    subtitle.set_color(app, Color::WHITE);
    subtitle.set_font_size(app, 20.0);
    let learn_label = Text::new(app, "Learn RetGui");
    learn_label.set_selectable(app, false);
    learn_label.set_color(app, palette::css::WHITE);
    let learn = Link(app, move |app, states| navigate("/docs", app, states));
    learn.set_padding(app, px(8), px(20), px(8), px(20));
    learn.set_background_color(app, rgb(69, 117, 230));
    learn.push(app, learn_label);
    let github_label = Text::new(app, "GitHub");
    github_label.set_selectable(app, false);
    github_label.set_color(app, palette::css::WHITE);
    let github = WebLink(app, "https://github.com/RetGui/retgui");
    github.set_padding(app, px(8), px(20), px(8), px(20));
    github.set_border_width_all(app, px(1));
    github.set_border_color_all(app, palette::css::WHITE);
    github.push(app, github_label);
    let buttons = Container::new(app);
    buttons.set_display(app, Display::Flex);
    buttons.set_wrap(app, FlexWrap::Wrap);
    buttons.set_gap(app, px(17), px(17));
    buttons.push(app, learn);
    buttons.push(app, github);
    let inner = wrapper(app);
    inner.set_display(app, Display::Flex);
    inner.set_flex_direction(app, FlexDirection::Column);
    inner.set_padding(
        app,
        Unit::Px(100.0),
        WRAPPER_PADDING_RIGHT,
        Unit::Px(100.0),
        WRAPPER_PADDING_LEFT,
    );
    inner.set_row_gap(app, px(28));
    inner.push(app, heading);
    inner.push(app, subtitle);
    inner.push(app, buttons);
    let container = Container::new(app);
    container.set_width(app, pct(100));
    container.set_background_color(app, rgb(45, 48, 53));
    container.push(app, inner);
    container
}

fn hero_features(app: &mut App) -> Container {
    let heading = Text::new(app, "Features");
    heading.set_width(app, pct(100));
    heading.set_font_size(app, 36.0);
    heading.set_font_weight(app, FontWeight::SEMIBOLD);
    let features = [
        (
            "Compile-time ownership",
            "Slotmap handles and explicit stores replace runtime borrow checks.",
        ),
        ("Pure Rust", "No UI macros are required."),
        ("Web-like styling", "Flexbox and block layout use familiar concepts."),
        (
            "Cross platform",
            "Windows, macOS, Linux, Web, and Android are supported.",
        ),
    ];
    let content = wrapper(app);
    content.set_padding(app, px(80), px(20), px(80), px(20));
    content.set_display(app, Display::Flex);
    content.set_wrap(app, FlexWrap::Wrap);
    content.set_gap(app, px(24), px(32));
    content.push(app, heading);
    for (name, description) in features {
        let name = Text::new(app, name);
        name.set_font_weight(app, FontWeight::MEDIUM);
        name.set_font_size(app, 24.0);
        let description = Text::new(app, description);
        description.set_font_size(app, 18.0);
        description.set_color(app, rgb(70, 70, 70));
        let item = Container::new(app);
        item.set_flex_grow(app, 1.0);
        item.set_min_width(app, px(320));
        item.set_flex_basis(app, pct(45));
        item.set_flex_direction(app, FlexDirection::Column);
        item.set_row_gap(app, px(8));
        item.push(app, name);
        item.push(app, description);
        content.push(app, item);
    }
    let container = Container::new(app);
    container.set_background_color(app, rgb(247, 247, 247));
    container.set_width(app, pct(100));
    container.push(app, content);
    container
}

pub(crate) fn index_page(app: &mut App, navigate: NavigateFn) -> Container {
    let intro = hero_intro(app, navigate);
    let features = hero_features(app);
    let page = Container::new(app);
    page.set_display(app, Display::Flex);
    page.set_width(app, pct(100));
    page.set_flex_direction(app, FlexDirection::Column);
    page.set_flex_grow(app, 1.0);
    page.push(app, intro);
    page.push(app, features);
    let container = Container::new(app);
    container.set_width(app, pct(100));
    container.set_overflow(app, Overflow::Visible, Overflow::Scroll);
    container.push(app, page);
    container
}
