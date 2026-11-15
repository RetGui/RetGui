use retgui::elements::Element;

mod constructors;
mod element;

#[cfg(feature = "audio")]
pub use constructors::audio;
#[cfg(feature = "code_highlighting")]
pub use constructors::code_editor;
pub use constructors::{button, calendar, checkbox, checkbox_group, container, dropdown, image, radio, radio_group, slider, text, text_input, tiny_vg, window};
pub use element::ElementBuilder;

pub struct Builder<E: Element> {
    element: E,
}

impl<E: Element> Builder<E> {
    pub fn new(element: E) -> Self {
        Self { element }
    }

    pub fn build(self) -> E {
        self.element
    }
}

impl<E: Element> From<E> for Builder<E> {
    fn from(element: E) -> Self {
        Self::new(element)
    }
}

pub trait IntoBuilder: Element {
    fn builder(self) -> Builder<Self> {
        Builder::new(self)
    }
}

impl<E: Element> IntoBuilder for E {}

impl<E: Element> ElementBuilder for Builder<E> {
    type Element = E;

    fn element(&self) -> E {
        self.element.clone()
    }
}

impl<E: Element> ElementBuilder for E {
    type Element = E;

    fn element(&self) -> E {
        self.clone()
    }
}

mod elements;
#[cfg(feature = "audio")]
pub use elements::AudioBuilder;
pub use elements::{CalendarBuilder, DropdownBuilder, ImageBuilder, RadioBuilder, SliderBuilder, TextBuilder, TextInputBuilder, TinyVgBuilder};

impl<E: Element> retgui::elements::AsElement for Builder<E> {
    fn with<R>(&self, callback: impl FnOnce(&dyn retgui::elements::ElementInternals) -> R) -> R {
        self.element.with(callback)
    }

    fn with_mut<R>(&self, callback: impl FnOnce(&mut dyn retgui::elements::ElementInternals) -> R) -> R {
        self.element.with_mut(callback)
    }
}
