use retgui::elements::Element;

mod element;
mod elements;

pub use element::ElementBuilder;
pub use elements::*;

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

pub trait IntoBuilder: Element {
    fn builder(self) -> Builder<Self> {
        Builder::new(self)
    }
}

impl<E: Element> IntoBuilder for E {}

impl<E: Element> ElementBuilder for Builder<E> {
    type Element = E;

    fn element(&self) -> E {
        self.element
    }
}

pub mod prelude {
    #[cfg(feature = "audio")]
    pub use crate::AudioBuilder;
    pub use crate::{CalendarBuilder, DropdownBuilder, ElementBuilder, ImageBuilder, IntoBuilder, RadioBuilder, RadioGroupBuilder, SliderBuilder, TextBuilder, TextInputBuilder, TinyVgBuilder, WindowBuilder};
}
