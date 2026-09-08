use crate::WebsiteState;

use retgui::App;
use retgui::elements::{Container, Element};
use retgui::events::PointerButton;

#[allow(non_snake_case)]
pub fn Link<F>(app: &mut App<WebsiteState>, on_click: F) -> Container
where
    F: Fn(&mut App<WebsiteState>, &mut WebsiteState) + 'static,
{
    let container = Container::new(app);
    container.add_pointer_button_up_listener(app, move |event, app, state| {
        if event.button == Some(PointerButton::Left) {
            on_click(app, state);
        }
    });
    container
}
