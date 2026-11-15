use std::any::Any;
use std::rc::Rc;

use retgui_primitives::Color;
use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::ElementBox;
use retgui_primitives::gradient::Gradient;

use smol_str::SmolStr;

use crate::RetGuiError;
use crate::app::queue_event;
use crate::elements::scrollable::{ScrollOptions, ScrollState};
use crate::elements::{AsElement, DynElement};
use crate::events::PointerId;
use crate::events::{CheckboxToggledEvent, ClickEvent, CustomEvent, EventCallbackKind, EventKind, EventListenerOptions, FocusEvent, KeyboardEvent, PointerButtonEvent, PointerCaptureEvent, PointerEnterEvent, PointerLeaveEvent, PointerMovedEvent, RadioValueChangedEvent, ScrollEvent, SliderValueChangedEvent, TextInputChangedEvent, UnfocusEvent};
use crate::style::{AlignContent, AlignItems, AlignSelf, Animation, BoxShadow, BoxSizing, Display, FlexDirection, FlexWrap, FontFamily, FontStyle, FontWeight, JustifyContent, Overflow, Position, ScrollbarColor, TextAlign, Underline, Unit};

pub trait Element: Clone + AsElement {
    /// Returns the element's children.
    fn get_children(&self) -> Vec<DynElement> {
        self.with(|element| element.get_children().to_vec())
    }

    /// Returns the element's previous sibling or a not found error.
    fn get_previous_sibling(&self) -> Result<DynElement, RetGuiError> {
        self.with(|element| element.get_previous_sibling())
    }

    /// Returns the element's next sibling or a not found error.
    fn get_next_sibling(&self) -> Result<DynElement, RetGuiError> {
        self.with(|element| element.get_next_sibling())
    }

    /// Returns the element's parent or a not found error.
    fn get_parent(&self) -> Result<DynElement, RetGuiError> {
        self.with(|element| element.parent())
            .ok_or(RetGuiError::ElementNotFound)
    }

    /// Returns the element's first child or a not found error.
    fn get_first_child(&self) -> Result<DynElement, RetGuiError> {
        self.with(|element| element.get_first_child())
    }

    /// Returns the element's last child or a not found error.
    fn get_last_child(&self) -> Result<DynElement, RetGuiError> {
        self.with(|element| element.get_last_child())
    }

    /// Removes the element's child or a not found error.
    fn remove_child(&self, child: DynElement) -> Result<DynElement, RetGuiError> {
        self.with_mut(|element| element.remove_child(child))
    }

    /// Removes the element's children.
    fn remove_all_children(&self) {
        self.with_mut(|element| element.remove_all_children())
    }

    /// Swaps the element's children or returns a not found error if either child is missing.
    fn swap_child(&self, child_1: DynElement, child_2: DynElement) -> Result<(), RetGuiError> {
        self.with_mut(|element| element.swap_child(child_1, child_2))
    }

    /// Pushes a child.
    fn push_child(&self, child: impl AsElement) {
        let child = child.with(|element| element.to_dyn_element());
        self.with_mut(|element| element.push(child));
    }

    /// Adds an event listener.
    fn listen_event(&self, callback: EventCallbackKind, options: EventListenerOptions) {
        self.with_mut(|element| element.add_event_listener(callback, options));
    }

    /// Adds a pointer enter listener.
    fn add_pointer_enter_listener(&self, on_pointer_enter: impl Fn(&mut PointerEnterEvent) + 'static) {
        self.with_mut(|element| element.on_pointer_enter(Rc::new(on_pointer_enter)));
    }

    /// Adds a pointer leave listener.
    fn add_pointer_leave_listener(&self, on_pointer_leave: impl Fn(&mut PointerLeaveEvent) + 'static) {
        self.with_mut(|element| element.on_pointer_leave(Rc::new(on_pointer_leave)));
    }

    /// Adds a radio value changed listener.
    fn add_radio_value_changed_listener(&self, on_radio_value_changed: impl Fn(&mut RadioValueChangedEvent) + 'static) {
        self.with_mut(|element| element.on_radio_value_changed(Rc::new(on_radio_value_changed)));
    }

    /// Adds a checkbox toggled listener.
    fn add_checkbox_toggled_listener(&self, on_checkbox_toggled: impl Fn(&mut CheckboxToggledEvent) + 'static) {
        self.with_mut(|element| element.on_checkbox_toggled(Rc::new(on_checkbox_toggled)));
    }

    /// Adds a text intput changed listener.
    fn add_text_input_changed_listener(&self, on_text_input_changed: impl Fn(&mut TextInputChangedEvent) + 'static) {
        self.with_mut(|element| element.on_text_input_changed(Rc::new(on_text_input_changed)));
    }

    /// Sets the element's user based id.
    fn set_id(&self, id: &str) {
        self.with_mut(|element| element.set_id(id));
    }

    /// Sets the accessibility name.
    fn set_accessibility_name(&self, name: &str) {
        self.with_mut(|element| element.element_data_mut().set_accessibility_name(name));
    }

    /// Returns the element's user based id. This id is not used by RetGUI.
    fn get_id(&self) -> Option<SmolStr> {
        self.with(|element| element.get_id())
    }

    /// Adds a pointer button down listener.
    fn add_pointer_button_down_listener(&self, on_pointer_button_down: impl Fn(&mut PointerButtonEvent) + 'static) {
        self.with_mut(|element| element.on_pointer_button_down(Rc::new(on_pointer_button_down)));
    }

    /// Adds a pointer button moved listener.
    fn add_pointer_moved_listener(&self, on_pointer_moved: impl Fn(&mut PointerMovedEvent) + 'static) {
        self.with_mut(|element| element.on_pointer_moved(Rc::new(on_pointer_moved)));
    }

    /// Adds a pointer button up listener.
    fn add_pointer_button_up_listener(&self, on_pointer_button_up: impl Fn(&mut PointerButtonEvent) + 'static) {
        self.with_mut(|element| element.on_pointer_button_up(Rc::new(on_pointer_button_up)));
    }

    /// Adds a click listener.
    fn add_click_listener(&self, on_click: impl Fn(&mut ClickEvent) + 'static) {
        self.with_mut(|element| element.on_click(Rc::new(on_click)));
    }

    /// Adds a custom event listener.
    fn add_custom_event_listener(&self, on_custom_event: impl Fn(&mut CustomEvent) + 'static) {
        self.with_mut(|element| element.on_custom_event(Rc::new(on_custom_event)));
    }

    /// Emits a custom event using the element as the target element.
    fn emit_custom_event<T: Any + 'static>(&self, detail: T) {
        queue_event(EventKind::Custom(CustomEvent::new(self.as_dyn_element(), detail)));
    }

    /// Adds a focus event listener.
    fn add_focus_listener(&self, on_focus: impl Fn(&mut FocusEvent) + 'static) {
        self.with_mut(|element| element.on_focus(Rc::new(on_focus)));
    }

    /// Adds an unfocus event listener.
    fn add_unfocus_listener(&self, on_unfocus: impl Fn(&mut UnfocusEvent) + 'static) {
        self.with_mut(|element| element.on_unfocus(Rc::new(on_unfocus)));
    }

    /// Adds a lost pointer capture event listener.
    fn add_lost_pointer_capture_listener(&self, on_lost_pointer_capture: impl Fn(&mut PointerCaptureEvent) + 'static) {
        self.with_mut(|element| element.on_lost_pointer_capture(Rc::new(on_lost_pointer_capture)));
    }

    /// Adds a got pointer capture event listener.
    fn add_got_pointer_capture_listener(&self, on_got_pointer_capture: impl Fn(&mut PointerCaptureEvent) + 'static) {
        self.with_mut(|element| element.on_got_pointer_capture(Rc::new(on_got_pointer_capture)));
    }

    /// Adds a keyboard input event listener.
    fn add_keyboard_input_listener(&self, on_keyboard_input: impl Fn(&mut KeyboardEvent) + 'static) {
        self.with_mut(|element| element.on_keyboard_input(Rc::new(on_keyboard_input)));
    }

    /// Adds a slider value changed event listener.
    fn add_slider_value_changed_listener(&self, on_slider_value_changed: impl Fn(&mut SliderValueChangedEvent) + 'static) {
        self.with_mut(|element| element.on_slider_value_changed(Rc::new(on_slider_value_changed)));
    }

    /// Adds a scroll event listener.
    fn add_scroll_listener(&self, on_scroll: impl Fn(&mut ScrollEvent) + 'static) {
        self.with_mut(|element| element.on_scroll(Rc::new(on_scroll)));
    }

    /// Scrolls to a child based on the child's user id.
    fn set_scroll_to_child_by_id(&self, id: &str) {
        self.with_mut(|element| element.scroll_to_child_by_id_with_options(id, ScrollOptions::default()));
    }

    /// Scrolls to a child based on the child's user id according to the scroll options.
    fn set_scroll_to_child_by_id_with_options(&self, id: &str, options: ScrollOptions) {
        self.with_mut(|element| element.scroll_to_child_by_id_with_options(id, options));
    }

    /// Scrolls to a specific y value in logical pixels.
    fn set_scroll_to(&self, y: f32) {
        self.with_mut(|element| element.scroll_to(y));
    }

    /// Scrolls to the top of the element.
    fn set_scroll_to_top(&self) {
        self.with_mut(|element| element.scroll_to_top());
    }

    /// Scrolls to the button of the element.
    fn set_scroll_to_bottom(&self) {
        self.with_mut(|element| element.scroll_to_bottom());
    }

    /// Scrolls by a logical amount of pixels.
    fn set_scroll_by(&self, y: f32) {
        self.with_mut(|element| element.scroll_by(y));
    }

    /// Returns the elements current scroll state.
    fn get_scroll_state(&self) -> ScrollState {
        self.with_mut(|element| element.get_scroll_state())
    }

    /// Sets the layout algorith e.g. block, flex, etc.
    fn set_display(&self, display: Display) {
        self.with_mut(|element| element.set_display(display));
    }

    /// Sets the box sizing e.g. content box/border box.
    fn set_box_sizing(&self, box_sizing: BoxSizing) {
        self.with_mut(|element| element.set_box_sizing(box_sizing));
    }

    /// Sets the position of the element.
    ///
    /// Unlike HTML, this has no effect on the visual order of the element.
    fn set_position(&self, position: Position) {
        self.with_mut(|element| element.set_position(position));
    }

    /// Puts the element on top of other elements.
    fn set_overlay(&self, overlay: bool) {
        self.with_mut(|element| element.set_overlay(overlay));
    }

    /// Returns if the element is put on top of other elements.
    fn get_overlay(&self) -> bool {
        self.with(|element| element.style().get_overlay())
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn set_margin(&self, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.with_mut(|element| element.set_margin(top, right, bottom, left));
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn set_margin_all(&self, value: Unit) {
        self.with_mut(|element| element.set_margin_all(value));
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn set_margin_vertical(&self, value: Unit) {
        self.with_mut(|element| element.set_margin_vertical(value));
    }

    /// Sets the non interactable/visual space surrounding the element.
    fn set_margin_horizontal(&self, value: Unit) {
        self.with_mut(|element| element.set_margin_horizontal(value));
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn set_padding(&self, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.with_mut(|element| element.set_padding(top, right, bottom, left));
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn set_padding_all(&self, value: Unit) {
        self.with_mut(|element| element.set_padding_all(value));
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn set_padding_vertical(&self, value: Unit) {
        self.with_mut(|element| element.set_padding_vertical(value));
    }

    /// Sets the interactable/visual space surrounding the element's content.
    fn set_padding_horizontal(&self, value: Unit) {
        self.with_mut(|element| element.set_padding_horizontal(value));
    }

    /// Sets the gap between children for flex/grid containers.
    fn set_gap(&self, row_gap: Unit, column_gap: Unit) {
        self.with_mut(|element| element.set_gap(row_gap, column_gap));
    }

    /// Sets the row gap between children for flex/grid containers.
    fn set_row_gap(&self, value: Unit) {
        self.with_mut(|element| element.set_row_gap(value));
    }

    /// Sets the column gap between children for flex/grid containers.
    fn set_column_gap(&self, value: Unit) {
        self.with_mut(|element| element.set_column_gap(value));
    }

    /// Align the element relative to its sides. Only applies to positioned elements.
    fn set_inset(&self, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.with_mut(|element| element.set_inset(top, right, bottom, left));
    }

    /// Sets the minium width of the element.
    fn set_min_width(&self, min_width: Unit) {
        self.with_mut(|element| element.set_min_width(min_width));
    }

    /// Sets the minium height of the element.
    fn set_min_height(&self, min_height: Unit) {
        self.with_mut(|element| element.set_min_height(min_height));
    }

    /// Sets the width of the element.
    fn set_width(&self, width: Unit) {
        self.with_mut(|element| element.set_width(width));
    }

    /// Sets the height of the element.
    fn set_height(&self, height: Unit) {
        self.with_mut(|element| element.set_height(height));
    }

    /// Sets the max width of the element.
    fn set_max_width(&self, max_width: Unit) {
        self.with_mut(|element| element.set_max_width(max_width));
    }

    /// Sets the max height of the element.
    fn set_max_height(&self, max_height: Unit) {
        self.with_mut(|element| element.set_max_height(max_height));
    }

    /// Sets the wrapping behavior for flex elements.
    fn set_wrap(&self, wrap: FlexWrap) {
        self.with_mut(|element| element.set_wrap(wrap));
    }

    /// Determines how flex/grid children are laid out on the cross axis.
    fn set_align_items(&self, align_items: AlignItems) {
        self.with_mut(|element| element.set_align_items(align_items));
    }

    /// Overrides a parent's align_items.
    fn set_align_self(&self, align_self: AlignSelf) {
        self.with_mut(|element| element.set_align_self(align_self));
    }

    fn set_align_content(&self, align_content: AlignContent) {
        self.with_mut(|element| element.set_align_content(align_content));
    }

    fn set_justify_content(&self, justify_content: JustifyContent) {
        self.with_mut(|element| element.set_justify_content(justify_content));
    }

    fn set_flex_direction(&self, flex_direction: FlexDirection) {
        self.with_mut(|element| element.set_flex_direction(flex_direction));
    }

    fn set_flex_grow(&self, flex_grow: f32) {
        self.with_mut(|element| element.set_flex_grow(flex_grow));
    }

    fn set_flex_shrink(&self, flex_shrink: f32) {
        self.with_mut(|element| element.set_flex_shrink(flex_shrink));
    }

    fn set_flex_basis(&self, flex_basis: Unit) {
        self.with_mut(|element| element.set_flex_basis(flex_basis));
    }

    fn set_order(&self, order: i32) {
        self.with_mut(|element| element.set_order(order));
    }

    fn set_font_family(&self, font_family: FontFamily) {
        self.with_mut(|element| element.set_font_family(font_family));
    }

    fn set_color(&self, color: Color) {
        self.with_mut(|element| element.set_text_brush(Brush::Color(color)));
    }

    fn set_text_gradient(&self, gradient: Gradient) {
        self.with_mut(|element| element.set_text_brush(Brush::Gradient(gradient)));
    }

    fn set_background_color(&self, background_color: Color) {
        self.with_mut(|element| element.set_background_brush(Brush::Color(background_color)));
    }

    fn set_background_gradient(&self, gradient: Gradient) {
        self.with_mut(|element| element.set_background_brush(Brush::Gradient(gradient)));
    }

    fn set_font_size(&self, font_size: f32) {
        self.with_mut(|element| element.set_font_size(font_size));
    }

    fn set_line_height(&self, line_height: f32) {
        self.with_mut(|element| element.set_line_height(line_height));
    }

    fn set_font_weight(&self, font_weight: FontWeight) {
        self.with_mut(|element| element.set_font_weight(font_weight));
    }

    fn set_font_style(&self, font_style: FontStyle) {
        self.with_mut(|element| element.set_font_style(font_style));
    }

    fn set_text_align(&self, text_align: TextAlign) {
        self.with_mut(|element| element.set_text_align(text_align));
    }

    fn set_underline(&self, thickness: Option<f32>, color: Color, offset: Option<f32>) {
        self.with_mut(|element| {
            element.set_underline(Some(Underline {
                thickness,
                brush: Brush::Color(color),
                offset,
            }))
        });

    }

    fn set_underline_gradient(&self, thickness: Option<f32>, gradient: Gradient, offset: Option<f32>) {
        self.with_mut(|element| {
            element.set_underline(Some(Underline {
                thickness,
                brush: Brush::Gradient(gradient),
                offset,
            }))
        });

    }

    fn set_overflow(&self, overflow_x: Overflow, overflow_y: Overflow) {
        self.with_mut(|element| element.set_overflow(overflow_x, overflow_y));
    }

    fn set_overflow_x(&self, overflow_x: Overflow) {
        self.with_mut(|element| element.set_overflow_x(overflow_x));
    }

    fn set_overflow_y(&self, overflow_y: Overflow) {
        self.with_mut(|element| element.set_overflow_y(overflow_y));
    }

    fn set_border_color(&self, top: Color, right: Color, bottom: Color, left: Color) {
        self.with_mut(|element| element.set_border_color(top, right, bottom, left));
    }

    fn set_border_color_all(&self, value: Color) {
        self.with_mut(|element| element.set_border_color_all(value));
    }

    fn set_border_color_vertical(&self, value: Color) {
        self.with_mut(|element| element.set_border_color_vertical(value));
    }

    fn set_border_color_horizontal(&self, value: Color) {
        self.with_mut(|element| element.set_border_color_horizontal(value));
    }

    fn set_border_width(&self, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.with_mut(|element| element.set_border_width(top, right, bottom, left));
    }

    fn set_border_width_all(&self, value: Unit) {
        self.with_mut(|element| element.set_border_width_all(value));
    }

    fn set_border_width_vertical(&self, value: Unit) {
        self.with_mut(|element| element.set_border_width_vertical(value));
    }

    fn set_border_width_horizontal(&self, value: Unit) {
        self.with_mut(|element| element.set_border_width_horizontal(value));
    }

    fn set_outline_color(&self, top: Color, right: Color, bottom: Color, left: Color) {
        self.with_mut(|element| element.set_outline_color(top, right, bottom, left));
    }

    fn set_outline_color_all(&self, value: Color) {
        self.with_mut(|element| element.set_outline_color_all(value));
    }

    fn set_outline_color_vertical(&self, value: Color) {
        self.with_mut(|element| element.set_outline_color_vertical(value));
    }

    fn set_outline_color_horizontal(&self, value: Color) {
        self.with_mut(|element| element.set_outline_color_horizontal(value));
    }

    fn set_outline_width(&self, top: Unit, right: Unit, bottom: Unit, left: Unit) {
        self.with_mut(|element| element.set_outline_width(top, right, bottom, left));
    }

    fn set_outline_width_all(&self, value: Unit) {
        self.with_mut(|element| element.set_outline_width_all(value));
    }

    fn set_outline_width_vertical(&self, value: Unit) {
        self.with_mut(|element| element.set_outline_width_vertical(value));
    }

    fn set_outline_width_horizontal(&self, value: Unit) {
        self.with_mut(|element| element.set_outline_width_horizontal(value));
    }

    fn set_border_radius(&self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) {
        self.with_mut(|element| element.set_border_radius(top, right, bottom, left));
    }

    fn set_border_radius_all(&self, value: (f32, f32)) {
        self.with_mut(|element| element.set_border_radius_all(value));
    }

    fn set_border_radius_vertical(&self, value: (f32, f32)) {
        self.with_mut(|element| element.set_border_radius_vertical(value));
    }

    fn set_border_radius_horizontal(&self, value: (f32, f32)) {
        self.with_mut(|element| element.set_border_radius_horizontal(value));
    }

    fn set_scrollbar_color(&self, scrollbar_color: ScrollbarColor) {
        self.with_mut(|element| element.set_scrollbar_color(scrollbar_color));
    }

    fn set_scrollbar_thumb_margin(&self, top: f32, right: f32, bottom: f32, left: f32) {
        self.with_mut(|element| element.set_scrollbar_thumb_margin(top, right, bottom, left));
    }

    fn set_scrollbar_thumb_radius(&self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) {
        self.with_mut(|element| element.set_scrollbar_thumb_radius(top, right, bottom, left));
    }

    fn set_scrollbar_width(&self, scrollbar_width: f32) {
        self.with_mut(|element| element.set_scrollbar_width(scrollbar_width));
    }

    /// Sets the list of animations.
    fn set_animations(&self, animations: Vec<Animation>) {
        self.with_mut(|element| element.set_animations(animations));
    }

    /// Sets the box shadows on this element.
    fn set_box_shadows(&self, box_shadows: Vec<BoxShadow>) {
        self.with_mut(|element| element.set_box_shadows(box_shadows));
    }

    /// Focus the element.
    fn focus_element(&self) {
        self.with_mut(|element| element.focus());
    }

    /// Returns whether the element current has focus.
    fn is_focused(&self) -> bool {
        self.with(|element| element.is_focused())
    }

    /// Unfocuses the element.
    fn unfocus_element(&self) {
        self.with_mut(|element| element.unfocus());
    }

    /// Get the elements box in logical pixels.
    fn get_computed_box_transformed(&self) -> ElementBox {
        self.with(|element| element.get_computed_box_transformed())
    }

    /// Returns whether the element has pointer capture.
    fn has_pointer_capture(&self, pointer_id: PointerId) -> bool {
        self.with(|element| element.has_pointer_capture(pointer_id))
    }

    /// Converts the element to a `DynElement`. This is useful when the type of element being stored does not matter.
    fn as_dyn_element(&self) -> DynElement {
        self.with(|element| element.to_dyn_element())
    }
}
