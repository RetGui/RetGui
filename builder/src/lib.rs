use retgui::App;
use retgui::elements::Element;

mod constructors;
mod element;
mod elements;

pub use constructors::*;
pub use element::ElementBuilder;
pub use elements::*;

pub type EventHandler<S, E> = fn(&mut E, &mut App<S>, &mut S);

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
        self.element
    }
}
