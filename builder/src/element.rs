use retgui::Color;

use retgui::Gradient;

use retgui::elements::{AsElement, ScrollOptions};
use retgui::events::{CheckboxToggledEvent, ClickEvent, CustomEvent, EventCallbackKind, EventListenerOptions, FocusEvent, KeyboardEvent, PointerButtonEvent, PointerCaptureEvent, PointerEnterEvent, PointerLeaveEvent, PointerMovedEvent, RadioValueChangedEvent, ScrollEvent, SliderValueChangedEvent, TextInputChangedEvent, UnfocusEvent};
use retgui::style::{AlignContent, AlignItems, AlignSelf, Animation, BoxShadow, BoxSizing, Display, FlexDirection, FlexWrap, FontFamily, FontStyle, FontWeight, JustifyContent, Overflow, Position, ScrollbarColor, TextAlign, Unit};

use retgui::elements::Element;

pub trait ElementBuilder: Sized {
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

    fn scroll_to_child_by_id(self, id: &str) -> Self {
        self.element().set_scroll_to_child_by_id(id);
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

    fn box_sizing(self, box_sizing: BoxSizing) -> Self {
        self.element().set_box_sizing(box_sizing);
        self
    }

    fn position(self, position: Position) -> Self {
        self.element().set_position(position);
        self
    }

    fn overlay(self, overlay: bool) -> Self {
        self.element().set_overlay(overlay);
        self
    }

    fn margin(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.element().set_margin(top, right, bottom, left);
        self
    }

    fn margin_all(self, value: Unit) -> Self {
        self.element().set_margin_all(value);
        self
    }

    fn margin_vertical(self, value: Unit) -> Self {
        self.element().set_margin_vertical(value);
        self
    }

    fn margin_horizontal(self, value: Unit) -> Self {
        self.element().set_margin_horizontal(value);
        self
    }

    fn padding(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.element().set_padding(top, right, bottom, left);
        self
    }

    fn padding_all(self, value: Unit) -> Self {
        self.element().set_padding_all(value);
        self
    }

    fn padding_vertical(self, value: Unit) -> Self {
        self.element().set_padding_vertical(value);
        self
    }

    fn padding_horizontal(self, value: Unit) -> Self {
        self.element().set_padding_horizontal(value);
        self
    }

    fn gap(self, row_gap: Unit, column_gap: Unit) -> Self {
        self.element().set_gap(row_gap, column_gap);
        self
    }

    fn row_gap(self, value: Unit) -> Self {
        self.element().set_row_gap(value);
        self
    }

    fn column_gap(self, value: Unit) -> Self {
        self.element().set_column_gap(value);
        self
    }

    fn inset(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.element().set_inset(top, right, bottom, left);
        self
    }

    fn min_width(self, min_width: Unit) -> Self {
        self.element().set_min_width(min_width);
        self
    }

    fn min_height(self, min_height: Unit) -> Self {
        self.element().set_min_height(min_height);
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

    fn max_width(self, max_width: Unit) -> Self {
        self.element().set_max_width(max_width);
        self
    }

    fn max_height(self, max_height: Unit) -> Self {
        self.element().set_max_height(max_height);
        self
    }

    fn wrap(self, wrap: FlexWrap) -> Self {
        self.element().set_wrap(wrap);
        self
    }

    fn align_items(self, align_items: AlignItems) -> Self {
        self.element().set_align_items(align_items);
        self
    }

    fn align_self(self, align_self: AlignSelf) -> Self {
        self.element().set_align_self(align_self);
        self
    }

    fn align_content(self, align_content: AlignContent) -> Self {
        self.element().set_align_content(align_content);
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

    fn flex_grow(self, flex_grow: f32) -> Self {
        self.element().set_flex_grow(flex_grow);
        self
    }

    fn flex_shrink(self, flex_shrink: f32) -> Self {
        self.element().set_flex_shrink(flex_shrink);
        self
    }

    fn flex_basis(self, flex_basis: Unit) -> Self {
        self.element().set_flex_basis(flex_basis);
        self
    }

    fn order(self, order: i32) -> Self {
        self.element().set_order(order);
        self
    }

    fn font_family(self, font_family: FontFamily) -> Self {
        self.element().set_font_family(font_family);
        self
    }

    fn color(self, color: Color) -> Self {
        self.element().set_color(color);
        self
    }

    fn text_gradient(self, gradient: Gradient) -> Self {
        self.element().set_text_gradient(gradient);
        self
    }

    fn background_color(self, background_color: Color) -> Self {
        self.element().set_background_color(background_color);
        self
    }

    fn background_gradient(self, gradient: Gradient) -> Self {
        self.element().set_background_gradient(gradient);
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

    fn font_weight(self, font_weight: FontWeight) -> Self {
        self.element().set_font_weight(font_weight);
        self
    }

    fn font_style(self, font_style: FontStyle) -> Self {
        self.element().set_font_style(font_style);
        self
    }

    fn text_align(self, text_align: TextAlign) -> Self {
        self.element().set_text_align(text_align);
        self
    }

    fn underline(self, thickness: Option<f32>, color: Color, offset: Option<f32>) -> Self {
        self.element().set_underline(thickness, color, offset);
        self
    }

    fn underline_gradient(self, thickness: Option<f32>, gradient: Gradient, offset: Option<f32>) -> Self {
        self.element().set_underline_gradient(thickness, gradient, offset);
        self
    }

    fn overflow(self, overflow_x: Overflow, overflow_y: Overflow) -> Self {
        self.element().set_overflow(overflow_x, overflow_y);
        self
    }

    fn overflow_x(self, overflow_x: Overflow) -> Self {
        self.element().set_overflow_x(overflow_x);
        self
    }

    fn overflow_y(self, overflow_y: Overflow) -> Self {
        self.element().set_overflow_y(overflow_y);
        self
    }

    fn border_color(self, top: Color, right: Color, bottom: Color, left: Color) -> Self {
        self.element().set_border_color(top, right, bottom, left);
        self
    }

    fn border_color_all(self, value: Color) -> Self {
        self.element().set_border_color_all(value);
        self
    }

    fn border_color_vertical(self, value: Color) -> Self {
        self.element().set_border_color_vertical(value);
        self
    }

    fn border_color_horizontal(self, value: Color) -> Self {
        self.element().set_border_color_horizontal(value);
        self
    }

    fn border_width(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.element().set_border_width(top, right, bottom, left);
        self
    }

    fn border_width_all(self, value: Unit) -> Self {
        self.element().set_border_width_all(value);
        self
    }

    fn border_width_vertical(self, value: Unit) -> Self {
        self.element().set_border_width_vertical(value);
        self
    }

    fn border_width_horizontal(self, value: Unit) -> Self {
        self.element().set_border_width_horizontal(value);
        self
    }

    fn outline_color(self, top: Color, right: Color, bottom: Color, left: Color) -> Self {
        self.element().set_outline_color(top, right, bottom, left);
        self
    }

    fn outline_color_all(self, value: Color) -> Self {
        self.element().set_outline_color_all(value);
        self
    }

    fn outline_color_vertical(self, value: Color) -> Self {
        self.element().set_outline_color_vertical(value);
        self
    }

    fn outline_color_horizontal(self, value: Color) -> Self {
        self.element().set_outline_color_horizontal(value);
        self
    }

    fn outline_width(self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Self {
        self.element().set_outline_width(top, right, bottom, left);
        self
    }

    fn outline_width_all(self, value: Unit) -> Self {
        self.element().set_outline_width_all(value);
        self
    }

    fn outline_width_vertical(self, value: Unit) -> Self {
        self.element().set_outline_width_vertical(value);
        self
    }

    fn outline_width_horizontal(self, value: Unit) -> Self {
        self.element().set_outline_width_horizontal(value);
        self
    }

    fn border_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self {
        self.element().set_border_radius(top, right, bottom, left);
        self
    }

    fn border_radius_all(self, value: (f32, f32)) -> Self {
        self.element().set_border_radius_all(value);
        self
    }

    fn border_radius_vertical(self, value: (f32, f32)) -> Self {
        self.element().set_border_radius_vertical(value);
        self
    }

    fn border_radius_horizontal(self, value: (f32, f32)) -> Self {
        self.element().set_border_radius_horizontal(value);
        self
    }

    fn scrollbar_color(self, scrollbar_color: ScrollbarColor) -> Self {
        self.element().set_scrollbar_color(scrollbar_color);
        self
    }

    fn scrollbar_thumb_margin(self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.element().set_scrollbar_thumb_margin(top, right, bottom, left);
        self
    }

    fn scrollbar_thumb_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self {
        self.element().set_scrollbar_thumb_radius(top, right, bottom, left);
        self
    }

    fn scrollbar_width(self, scrollbar_width: f32) -> Self {
        self.element().set_scrollbar_width(scrollbar_width);
        self
    }

    fn animations(self, animations: Vec<Animation>) -> Self {
        self.element().set_animations(animations);
        self
    }

    fn box_shadows(self, box_shadows: Vec<BoxShadow>) -> Self {
        self.element().set_box_shadows(box_shadows);
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
