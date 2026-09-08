use std::rc::Rc;

use retgui::elements::{Container, Element, Text};
use retgui::events::PointerButton;
use retgui::style::{Display, FlexDirection, FontWeight, Overflow};
use retgui::{App, palette, pct, px};

use crate::WebsiteState;
use crate::router::NavigateFn;
use crate::theme::{ACTIVE_LINK_COLOR, DEFAULT_LINK_COLOR, wrapper};

#[allow(dead_code)]
#[path = "../../examples/counter/main.rs"]
pub mod counter;
#[allow(dead_code)]
#[path = "../../examples/pointer_events/main.rs"]
mod pointer_events;
#[allow(dead_code)]
#[path = "../../examples/text/main.rs"]
mod text;

const COUNTER: &str = "/examples/counter";
const POINTER_EVENTS: &str = "/examples/pointer-events";
const TEXT: &str = "/examples/text";

fn show_example(app: &mut App<WebsiteState>, examples: &[Container], selected: usize) {
    for (index, example) in examples.iter().enumerate() {
        example.set_display(
            app,
            if index == selected {
                Display::Flex
            } else {
                Display::None
            },
        );
    }
}

fn example_link(
    app: &mut App<WebsiteState>,
    label: &str,
    route: &'static str,
    index: usize,
    selected: usize,
    examples: Rc<Vec<Container>>,
    navigate: NavigateFn,
) -> Text {
    let color = if selected == index {
        ACTIVE_LINK_COLOR
    } else {
        DEFAULT_LINK_COLOR
    };
    let text = Text::new(app, label);
    text.set_color(app, color);
    text.set_selectable(app, false);
    text.add_pointer_button_up_listener(app, move |event, app, state| {
        if event.button == Some(PointerButton::Left) {
            state.selected_example = index;
            show_example(app, &examples, state.selected_example);
            navigate(route, app, state);
        }
    });
    text
}

pub fn examples(app: &mut App<WebsiteState>, route: &str, navigate: NavigateFn) -> (Container, usize) {
    let counter = counter::counter(app, |state| &mut state.counter);
    counter.set_id(app, COUNTER);
    let pointer = pointer_events::pointer_events(app);
    pointer.set_id(app, POINTER_EVENTS);
    let text = text::text(app);
    text.set_id(app, TEXT);
    let examples = Rc::new(vec![counter, pointer, text]);
    let selected_index = [COUNTER, POINTER_EVENTS, TEXT]
        .iter()
        .position(|candidate| *candidate == route)
        .unwrap_or(0);
    show_example(app, &examples, selected_index);

    let heading = Text::new(app, "Examples");
    heading.set_selectable(app, false);
    heading.set_font_weight(app, FontWeight::MEDIUM);
    heading.set_font_size(app, 20.0);
    let sidebar = Container::new(app);
    sidebar.set_display(app, Display::Flex);
    sidebar.set_flex_direction(app, FlexDirection::Column);
    sidebar.set_gap(app, px(12), px(12));
    sidebar.set_min_width(app, px(210));
    sidebar.push(app, heading);
    for (index, (label, route)) in [("Counter", COUNTER), ("Pointer events", POINTER_EVENTS), ("Text", TEXT)]
        .into_iter()
        .enumerate()
    {
        let link = example_link(app, label, route, index, selected_index, examples.clone(), navigate);
        sidebar.push(app, link);
    }

    let content = Container::new(app);
    content.set_width(app, pct(100));
    content.set_height(app, px(600));
    content.set_background_color(app, palette::css::WHITE);
    for example in examples.iter() {
        content.push(app, *example);
    }
    let page = wrapper(app);
    page.set_padding_all(app, px(40));
    page.set_gap(app, px(24), px(24));
    page.push(app, sidebar);
    page.push(app, content);
    let container = Container::new(app);
    container.set_overflow(app, Overflow::Visible, Overflow::Scroll);
    container.push(app, page);
    (container, selected_index)
}
