#[cfg(feature = "audio")]
use retgui::elements::Audio;
use retgui::elements::{Calendar, Dropdown, Image, Radio, Slider, SliderDirection, Text, TextInput, TinyVg};
use retgui::events::CalendarDateSelectedEvent;
use retgui::text::RangedStyles;
use retgui::{Brush, Color, Gradient, ResourceId};

use crate::ElementBuilder;

pub trait TextBuilder: ElementBuilder<Element = Text> {
    fn selectable(self, selectable: bool) -> Self {
        self.element().set_selectable(selectable);
        self
    }
    fn text(self, text: &str) -> Self {
        self.element().set_text(text);
        self
    }
}
impl<B: ElementBuilder<Element = Text>> TextBuilder for B {}

pub trait TextInputBuilder: ElementBuilder<Element = TextInput> {
    fn disabled(self, disabled: bool) -> Self {
        self.element().set_disabled(disabled);
        self
    }
    fn multiline(self, multiline: bool) -> Self {
        self.element().set_multiline(multiline);
        self
    }
    fn text(self, text: &str) -> Self {
        self.element().set_text(text);
        self
    }
    fn ranged_styles(self, ranged_styles: RangedStyles) -> Self {
        self.element().set_ranged_styles(ranged_styles);
        self
    }
}
impl<B: ElementBuilder<Element = TextInput>> TextInputBuilder for B {}

pub trait SliderBuilder: ElementBuilder<Element = Slider> {
    fn value(self, value: f64) -> Self {
        self.element().set_value(value);
        self
    }
    fn step(self, value: f64) -> Self {
        self.element().set_step(value);
        self
    }
    fn min(self, min: f64) -> Self {
        self.element().set_min(min);
        self
    }
    fn max(self, max: f64) -> Self {
        self.element().set_max(max);
        self
    }
    fn direction(self, direction: SliderDirection) -> Self {
        self.element().set_direction(direction);
        self
    }
    fn thumb_size(self, thumb_size: f64) -> Self {
        self.element().set_thumb_size(thumb_size);
        self
    }
    fn thumb_color(self, thumb_background_color: Brush) -> Self {
        self.element().set_thumb_color(thumb_background_color);
        self
    }
    fn thumb_border_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self {
        self.element().set_thumb_border_radius(top, right, bottom, left);
        self
    }
    fn track_color(self, track_background_color: Color) -> Self {
        self.element().set_track_color(track_background_color);
        self
    }
    fn track_gradient(self, track_background_gradient: Gradient) -> Self {
        self.element().set_track_gradient(track_background_gradient);
        self
    }
    fn track_border_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self {
        self.element().set_track_border_radius(top, right, bottom, left);
        self
    }
}
impl<B: ElementBuilder<Element = Slider>> SliderBuilder for B {}

pub trait ImageBuilder: ElementBuilder<Element = Image> {
    fn resource_id(self, resource_id: ResourceId) -> Self {
        self.element().set_resource_id(resource_id);
        self
    }
}
impl<B: ElementBuilder<Element = Image>> ImageBuilder for B {}

pub trait TinyVgBuilder: ElementBuilder<Element = TinyVg> {
    fn resource_id(self, resource_id: ResourceId) -> Self {
        self.element().set_resource_id(resource_id);
        self
    }
}
impl<B: ElementBuilder<Element = TinyVg>> TinyVgBuilder for B {}

pub trait DropdownBuilder: ElementBuilder<Element = Dropdown> {
    fn selected_item(self, index: usize) -> Self {
        self.element().set_selected_item(index);
        self
    }
}
impl<B: ElementBuilder<Element = Dropdown>> DropdownBuilder for B {}

pub trait CalendarBuilder: ElementBuilder<Element = Calendar> {
    fn on_date_selected(self, handler: impl Fn(&mut CalendarDateSelectedEvent) + 'static) -> Self {
        self.element().add_date_selected_listener(handler);
        self
    }
    fn start_year(self, year: i32) -> Self {
        self.element().set_start_year(year);
        self
    }
    fn end_year(self, year: i32) -> Self {
        self.element().set_end_year(year);
        self
    }
}
impl<B: ElementBuilder<Element = Calendar>> CalendarBuilder for B {}

#[cfg(feature = "audio")]
pub trait AudioBuilder: ElementBuilder<Element = Audio> {
    fn controls(self, controls: bool) -> Self {
        self.element().set_controls(controls);
        self
    }
}
#[cfg(feature = "audio")]
impl<B: ElementBuilder<Element = Audio>> AudioBuilder for B {}

pub trait RadioBuilder: ElementBuilder<Element = Radio> {
    fn hide_radio(self) -> Self {
        self.element().set_hide_radio(true);
        self
    }
}
impl<B: ElementBuilder<Element = Radio>> RadioBuilder for B {}
