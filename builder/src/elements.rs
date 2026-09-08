use std::sync::Arc;

#[cfg(feature = "audio")]
use retgui::elements::Audio;
use retgui::elements::{Calendar, Dropdown, Image, Radio, RadioGroup, Slider, SliderDirection, Text, TextInput, TinyVg, Window};
use retgui::text::RangedStyles;
use retgui::winit::window::Window as WinitWindow;
use retgui::{App, Brush, Color, Gradient, ResourceId};
use smol_str::SmolStr;

use crate::ElementBuilder;

pub trait TextBuilder: ElementBuilder<Element = Text> {
    fn selectable<S: 'static>(self, app: &mut App<S>, selectable: bool) -> Self {
        self.element().set_selectable(app, selectable);
        self
    }

    fn text<S: 'static>(self, app: &mut App<S>, text: &str) -> Self {
        self.element().set_text(app, text);
        self
    }

    fn text_smol_str<S: 'static>(self, app: &mut App<S>, text: SmolStr) -> Self {
        self.element().set_text_smol_str(app, text);
        self
    }
}

impl<B: ElementBuilder<Element = Text>> TextBuilder for B {}

pub trait TextInputBuilder: ElementBuilder<Element = TextInput> {
    fn disabled<S: 'static>(self, app: &mut App<S>, disabled: bool) -> Self {
        self.element().set_disabled(app, disabled);
        self
    }

    fn multiline<S: 'static>(self, app: &mut App<S>, multiline: bool) -> Self {
        self.element().set_multiline(app, multiline);
        self
    }

    fn text<S: 'static>(self, app: &mut App<S>, text: &str) -> Self {
        self.element().set_text(app, text);
        self
    }

    fn ranged_styles<S: 'static>(self, app: &mut App<S>, ranged_styles: RangedStyles) -> Self {
        self.element().set_ranged_styles(app, ranged_styles);
        self
    }
}

impl<B: ElementBuilder<Element = TextInput>> TextInputBuilder for B {}

pub trait SliderBuilder: ElementBuilder<Element = Slider> {
    fn value<S: 'static>(self, app: &mut App<S>, value: f64) -> Self {
        self.element().set_value(app, value);
        self
    }

    fn step<S: 'static>(self, app: &mut App<S>, value: f64) -> Self {
        self.element().set_step(app, value);
        self
    }

    fn min<S: 'static>(self, app: &mut App<S>, min: f64) -> Self {
        self.element().set_min(app, min);
        self
    }

    fn max<S: 'static>(self, app: &mut App<S>, max: f64) -> Self {
        self.element().set_max(app, max);
        self
    }

    fn direction<S: 'static>(self, app: &mut App<S>, direction: SliderDirection) -> Self {
        self.element().set_direction(app, direction);
        self
    }

    fn thumb_size<S: 'static>(self, app: &mut App<S>, thumb_size: f64) -> Self {
        self.element().set_thumb_size(app, thumb_size);
        self
    }

    fn thumb_color<S: 'static>(self, app: &mut App<S>, thumb_background_color: Brush) -> Self {
        self.element().set_thumb_color(app, thumb_background_color);
        self
    }

    fn thumb_border_radius<S: 'static>(
        self,
        app: &mut App<S>,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) -> Self {
        self.element().set_thumb_border_radius(app, top, right, bottom, left);
        self
    }

    fn track_color<S: 'static>(self, app: &mut App<S>, track_background_color: Color) -> Self {
        self.element().set_track_color(app, track_background_color);
        self
    }

    fn track_gradient<S: 'static>(self, app: &mut App<S>, track_background_gradient: Gradient) -> Self {
        self.element().set_track_gradient(app, track_background_gradient);
        self
    }

    fn track_border_radius<S: 'static>(
        self,
        app: &mut App<S>,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) -> Self {
        self.element().set_track_border_radius(app, top, right, bottom, left);
        self
    }
}

impl<B: ElementBuilder<Element = Slider>> SliderBuilder for B {}

pub trait ImageBuilder: ElementBuilder<Element = Image> {
    fn resource_id<S: 'static>(self, app: &mut App<S>, resource_id: ResourceId) -> Self {
        self.element().set_resource_id(app, resource_id);
        self
    }
}

impl<B: ElementBuilder<Element = Image>> ImageBuilder for B {}

pub trait TinyVgBuilder: ElementBuilder<Element = TinyVg> {
    fn resource_id<S: 'static>(self, app: &mut App<S>, resource_id: ResourceId) -> Self {
        self.element().set_resource_id(app, resource_id);
        self
    }
}

impl<B: ElementBuilder<Element = TinyVg>> TinyVgBuilder for B {}

pub trait CalendarBuilder: ElementBuilder<Element = Calendar> {
    fn start_year<S: 'static>(self, app: &mut App<S>, year: i32) -> Self {
        self.element().set_start_year(app, year);
        self
    }

    fn end_year<S: 'static>(self, app: &mut App<S>, year: i32) -> Self {
        self.element().set_end_year(app, year);
        self
    }
}

impl<B: ElementBuilder<Element = Calendar>> CalendarBuilder for B {}

pub trait DropdownBuilder: ElementBuilder<Element = Dropdown> {
    fn selected_item<S: 'static>(self, app: &mut App<S>, index: usize) -> Self {
        self.element().set_selected_item(app, index);
        self
    }
}

impl<B: ElementBuilder<Element = Dropdown>> DropdownBuilder for B {}

pub trait RadioBuilder: ElementBuilder<Element = Radio> {
    fn hide_radio<S: 'static>(self, app: &mut App<S>, value: bool) -> Self {
        self.element().set_hide_radio(app, value);
        self
    }
}

impl<B: ElementBuilder<Element = Radio>> RadioBuilder for B {}

pub trait RadioGroupBuilder: ElementBuilder<Element = RadioGroup> {
    fn value<S: 'static>(self, app: &mut App<S>, value: &str) -> Self {
        self.element().set_value(app, value);
        self
    }
}

impl<B: ElementBuilder<Element = RadioGroup>> RadioGroupBuilder for B {}

pub trait WindowBuilder: ElementBuilder<Element = Window> {
    fn winit_window<S: 'static>(self, app: &mut App<S>, window: Option<Arc<dyn WinitWindow>>) -> Self {
        self.element().set_winit_window(app, window);
        self
    }

    fn scale_factor<S: 'static>(self, app: &mut App<S>, scale_factor: f64) -> Self {
        self.element().set_scale_factor(app, scale_factor);
        self
    }
}

impl<B: ElementBuilder<Element = Window>> WindowBuilder for B {}

#[cfg(feature = "audio")]
pub trait AudioBuilder: ElementBuilder<Element = Audio> {
    fn controls<S: 'static>(self, app: &mut App<S>, controls: bool) -> Self {
        self.element().set_controls(app, controls);
        self
    }
}

#[cfg(feature = "audio")]
impl<B: ElementBuilder<Element = Audio>> AudioBuilder for B {}
