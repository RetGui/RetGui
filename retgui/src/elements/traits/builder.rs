use retgui_primitives::Color;


use crate::elements::AsElement;
use crate::elements::scrollable::ScrollOptions;
use crate::events::{CheckboxToggledEvent, ClickEvent, CustomEvent, EventCallbackKind, EventListenerOptions, FocusEvent, KeyboardEvent, PointerButtonEvent, PointerCaptureEvent, PointerEnterEvent, PointerLeaveEvent, PointerMovedEvent, RadioValueChangedEvent, ScrollEvent, SliderValueChangedEvent, TextInputChangedEvent, UnfocusEvent};
use crate::style::{AlignItems, Display, FlexDirection, JustifyContent, Position, Unit};

use crate::elements::Element;

pub(crate) trait ElementBuilder: Sized {
    type Element: Element;
    fn element(&self) -> Self::Element;

    fn push(self, child: impl AsElement) -> Self {
        self.element().push_child(child);
        self
    }

    fn add_event_listener(self, callback: EventCallbackKind, options: EventListenerOptions) -> Self {
        self.element().listen_event(callback, options);
        self
    }

    fn on_pointer_enter(self, on_pointer_enter: impl Fn(&mut PointerEnterEvent) + 'static) -> Self {
        self.element().add_pointer_enter_listener(on_pointer_enter);
        self
    }

    fn on_pointer_leave(self, on_pointer_leave: impl Fn(&mut PointerLeaveEvent) + 'static) -> Self {
        self.element().add_pointer_leave_listener(on_pointer_leave);
        self
    }

    fn on_radio_value_changed(self, on_radio_value_changed: impl Fn(&mut RadioValueChangedEvent) + 'static) -> Self {
        self.element().add_radio_value_changed_listener(on_radio_value_changed);
        self
    }

    fn on_checkbox_toggled(self, on_checkbox_toggled: impl Fn(&mut CheckboxToggledEvent) + 'static) -> Self {
        self.element().add_checkbox_toggled_listener(on_checkbox_toggled);
        self
    }

    fn on_text_input_changed(self, on_text_input_changed: impl Fn(&mut TextInputChangedEvent) + 'static) -> Self {
        self.element().add_text_input_changed_listener(on_text_input_changed);
        self
    }

    fn id(self, id: &str) -> Self {
        self.element().set_id(id);
        self
    }

    fn accessibility_name(self, name: &str) -> Self {
        self.element().set_accessibility_name(name);
        self
    }

    fn on_pointer_button_down(self, on_pointer_button_down: impl Fn(&mut PointerButtonEvent) + 'static) -> Self {
        self.element().add_pointer_button_down_listener(on_pointer_button_down);
        self
    }

    fn on_pointer_moved(self, on_pointer_moved: impl Fn(&mut PointerMovedEvent) + 'static) -> Self {
        self.element().add_pointer_moved_listener(on_pointer_moved);
        self
    }

    fn on_pointer_button_up(self, on_pointer_button_up: impl Fn(&mut PointerButtonEvent) + 'static) -> Self {
        self.element().add_pointer_button_up_listener(on_pointer_button_up);
        self
    }

    fn on_click(self, on_click: impl Fn(&mut ClickEvent) + 'static) -> Self {
        self.element().add_click_listener(on_click);
        self
    }

    fn on_custom_event(self, on_custom_event: impl Fn(&mut CustomEvent) + 'static) -> Self {
        self.element().add_custom_event_listener(on_custom_event);
        self
    }

    fn on_focus(self, on_focus: impl Fn(&mut FocusEvent) + 'static) -> Self {
        self.element().add_focus_listener(on_focus);
        self
    }

    fn on_unfocus(self, on_unfocus: impl Fn(&mut UnfocusEvent) + 'static) -> Self {
        self.element().add_unfocus_listener(on_unfocus);
        self
    }

    fn on_lost_pointer_capture(self, on_lost_pointer_capture: impl Fn(&mut PointerCaptureEvent) + 'static) -> Self {
        self.element()
            .add_lost_pointer_capture_listener(on_lost_pointer_capture);
        self
    }

    fn on_got_pointer_capture(self, on_got_pointer_capture: impl Fn(&mut PointerCaptureEvent) + 'static) -> Self {
        self.element().add_got_pointer_capture_listener(on_got_pointer_capture);
        self
    }

    fn on_keyboard_input(self, on_keyboard_input: impl Fn(&mut KeyboardEvent) + 'static) -> Self {
        self.element().add_keyboard_input_listener(on_keyboard_input);
        self
    }

    fn on_slider_value_changed(self, on_slider_value_changed: impl Fn(&mut SliderValueChangedEvent) + 'static) -> Self {
        self.element()
            .add_slider_value_changed_listener(on_slider_value_changed);
        self
    }

    fn on_scroll(self, on_scroll: impl Fn(&mut ScrollEvent) + 'static) -> Self {
        self.element().add_scroll_listener(on_scroll);
        self
    }

    fn scroll_to_child_by_id_with_options(self, id: &str, options: ScrollOptions) -> Self {
        self.element().set_scroll_to_child_by_id_with_options(id, options);
        self
    }

    fn scroll_to(self, y: f32) -> Self {
        self.element().set_scroll_to(y);
        self
    }

    fn scroll_to_top(self) -> Self {
        self.element().set_scroll_to_top();
        self
    }

    fn scroll_to_bottom(self) -> Self {
        self.element().set_scroll_to_bottom();
        self
    }

    fn scroll_by(self, y: f32) -> Self {
        self.element().set_scroll_by(y);
        self
    }

    fn display(self, display: Display) -> Self {
        self.element().set_display(display);
        self
    }

    fn position(self, position: Position) -> Self {
        self.element().set_position(position);
        self
    }

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

    fn outline_color_all(self, value: Color) -> Self {
        self.element().set_outline_color_all(value);
        self
    }

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
