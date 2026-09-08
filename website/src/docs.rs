use crate::WebsiteState;

use retgui::elements::{Container, Element};
use retgui::style::{Display, FlexDirection, Overflow, Unit};
use retgui::{App, pct};

use crate::router::NavigateFn;

pub(crate) fn docs(app: &mut App<WebsiteState>, _navigate_fn: NavigateFn) -> Container {
    let content = Container::new(app);
    content.set_display(app, Display::Flex);
    content.set_width(app, pct(100));
    content.set_margin(app, Unit::Px(0.0), Unit::Auto, Unit::Px(0.0), Unit::Auto);
    content.set_flex_direction(app, FlexDirection::Column);
    content.set_flex_grow(app, 1.0);
    let container = Container::new(app);
    container.set_width(app, pct(100));
    container.set_overflow(app, Overflow::Visible, Overflow::Scroll);
    container.push(app, content);
    container
}
