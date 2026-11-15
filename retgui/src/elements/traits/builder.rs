use retgui_primitives::Color;

use crate::elements::AsElement;
use crate::events::{EventCallbackKind, EventListenerOptions};
use crate::style::{AlignItems, Display, FlexDirection, JustifyContent, Unit};

use crate::elements::Element;
#[cfg(test)]
use crate::events::PointerButtonEvent;
#[cfg(feature = "audio")]
use crate::events::{ClickEvent, SliderValueChangedEvent};

pub(crate) trait ElementBuilder: Sized {
    type Element: Element;
    fn element(&self) -> Self::Element;

    #[cfg(test)]
    fn on_pointer_button_up(self, on_pointer_button_up: impl Fn(&mut PointerButtonEvent) + 'static) -> Self {
        self.element().add_pointer_button_up_listener(on_pointer_button_up);
        self
    }

    fn push(self, child: impl AsElement) -> Self {
        self.element().push_child(child);
        self
    }

    fn add_event_listener(self, callback: EventCallbackKind, options: EventListenerOptions) -> Self {
        self.element().listen_event(callback, options);
        self
    }

    #[cfg(any(feature = "audio", test))]
    fn accessibility_name(self, name: &str) -> Self {
        self.element().set_accessibility_name(name);
        self
    }

    #[cfg(feature = "audio")]
    fn on_click(self, on_click: impl Fn(&mut ClickEvent) + 'static) -> Self {
        self.element().add_click_listener(on_click);
        self
    }

    #[cfg(feature = "audio")]
    fn on_slider_value_changed(self, on_slider_value_changed: impl Fn(&mut SliderValueChangedEvent) + 'static) -> Self {
        self.element()
            .add_slider_value_changed_listener(on_slider_value_changed);
        self
    }

    fn display(self, display: Display) -> Self {
        self.element().set_display(display);
        self
    }

    #[cfg(feature = "markdown")]
    fn margin(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.element().set_margin(top, right, bottom, left);
        self
    }

    fn width(self, width: Unit) -> Self {
        self.element().set_width(width);
        self
    }

    fn height(self, height: Unit) -> Self {
        self.element().set_height(height);
        self
    }

    fn align_items(self, align_items: AlignItems) -> Self {
        self.element().set_align_items(align_items);
        self
    }

    fn justify_content(self, justify_content: JustifyContent) -> Self {
        self.element().set_justify_content(justify_content);
        self
    }

    fn flex_direction(self, flex_direction: FlexDirection) -> Self {
        self.element().set_flex_direction(flex_direction);
        self
    }

    fn color(self, color: Color) -> Self {
        self.element().set_color(color);
        self
    }

    fn background_color(self, background_color: Color) -> Self {
        self.element().set_background_color(background_color);
        self
    }

    fn font_size(self, font_size: f32) -> Self {
        self.element().set_font_size(font_size);
        self
    }

    #[cfg(feature = "audio")]
    fn line_height(self, line_height: f32) -> Self {
        self.element().set_line_height(line_height);
        self
    }

    fn border_color_all(self, value: Color) -> Self {
        self.element().set_border_color_all(value);
        self
    }

    fn border_width_all(self, value: Unit) -> Self {
        self.element().set_border_width_all(value);
        self
    }

    #[cfg(test)]
    fn outline_color_all(self, value: Color) -> Self {
        self.element().set_outline_color_all(value);
        self
    }

    #[cfg(test)]
    fn outline_width_all(self, value: Unit) -> Self {
        self.element().set_outline_width_all(value);
        self
    }

    fn border_radius_all(self, value: (f32, f32)) -> Self {
        self.element().set_border_radius_all(value);
        self
    }

    fn focus(self) -> Self {
        self.element().focus_element();
        self
    }

    fn unfocus(self) -> Self {
        self.element().unfocus_element();
        self
    }
}

impl<E: Element> ElementBuilder for E {
    type Element = E;

    fn element(&self) -> E {
        self.clone()
    }
}
