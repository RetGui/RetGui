use std::rc::Rc;

use retgui::elements::{Container, Element};
use retgui::events::PointerButton;
use retgui::{App, States};

#[allow(non_snake_case)]
pub fn Link<F>(app: &mut App, on_click: F) -> Container
where
    F: Fn(&mut App, &mut States) + 'static,
{
    let on_click = Rc::new(on_click);

    let container = Container::new(app);
    container.add_pointer_button_up_listener(app, move |event, app, states| {
        if event.button == Some(PointerButton::Left) {
            on_click(app, states);
        }
    });
    container
}
