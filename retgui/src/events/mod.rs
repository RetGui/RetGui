//! Typed events and event dispatch controls.
//!
//! Import [`Event`] to use the behavior shared by every concrete event type,
//! such as [`Event::stop_propagation`] and [`Event::prevent_default`].

pub use winit::event::{
    ElementState, Ime, Modifiers, MouseButton, MouseButton as PointerButton, MouseScrollDelta as ScrollDelta,
    PointerKind as PointerId,
};
pub use winit::keyboard::{
    Key, KeyCode as Code, KeyLocation as Location, ModifiersState as KeyboardModifiers, NamedKey,
};

pub use crate::events::mouse_wheel::MouseWheel;
pub(crate) use event_dispatch::EventDispatcher;

use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;

use retgui_primitives::geometry::Point;

use winit::dpi::{LogicalPosition, PhysicalPosition};
use winit::event::{ButtonSource, KeyEvent, PointerKind, PointerSource};

use crate::App;
use crate::elements::DynElement;

pub mod pointer_capture;

mod event_dispatch;
mod helpers;
mod mouse_wheel;

/// The broad class of device that generated a pointer event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointerType {
    Mouse,
    Pen,
    Touch,
    #[default]
    Unknown,
}

/// Stable identifying information for a pointer interaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerInfo {
    pub pointer_id: Option<PointerId>,
    pub pointer_type: PointerType,
    primary: bool,
}

impl PointerInfo {
    pub(crate) fn new(pointer_id: PointerId, primary: bool) -> Self {
        let pointer_type = match pointer_id {
            PointerKind::Mouse => PointerType::Mouse,
            PointerKind::Touch(_) => PointerType::Touch,
            PointerKind::TabletTool(_) => PointerType::Pen,
            PointerKind::Unknown => PointerType::Unknown,
            _ => PointerType::Unknown,
        };
        Self {
            pointer_id: Some(pointer_id),
            pointer_type,
            primary,
        }
    }

    pub fn is_primary_pointer(&self) -> bool {
        self.primary
    }

    pub(crate) fn from_source(source: &PointerSource, primary: bool) -> Self {
        let pointer_id = match source {
            PointerSource::Mouse => PointerKind::Mouse,
            PointerSource::Touch { finger_id, .. } => PointerKind::Touch(*finger_id),
            PointerSource::TabletTool { kind, .. } => PointerKind::TabletTool(*kind),
            PointerSource::Unknown => PointerKind::Unknown,
            _ => PointerKind::Unknown,
        };
        Self::new(pointer_id, primary)
    }

    pub(crate) fn from_button(source: &ButtonSource, primary: bool) -> Self {
        let pointer_id = match source {
            ButtonSource::Mouse(_) => PointerKind::Mouse,
            ButtonSource::Touch { finger_id, .. } => PointerKind::Touch(*finger_id),
            ButtonSource::TabletTool { kind, .. } => PointerKind::TabletTool(*kind),
            ButtonSource::Unknown(_) => PointerKind::Unknown,
            _ => PointerKind::Unknown,
        };
        Self::new(pointer_id, primary)
    }
}

/// Position and scale information attached to a pointer event.
#[derive(Clone, Debug, PartialEq)]
pub struct PointerState {
    pub position: PhysicalPosition<f64>,
    pub scale_factor: f64,
}

impl PointerState {
    pub(crate) fn new(position: PhysicalPosition<f64>, scale_factor: f64) -> Self {
        Self {
            position,
            scale_factor,
        }
    }

    pub fn logical_point(&self) -> Point {
        let position = self.logical_position();
        Point::new(position.x, position.y)
    }

    pub fn logical_position(&self) -> LogicalPosition<f64> {
        self.position.to_logical(self.scale_factor)
    }
}

/// State shared by every RetGui event.
#[derive(Clone)]
pub struct BaseEvent {
    target: DynElement,
    current_target: DynElement,
    propagation_stopped: bool,
    default_prevented: bool,
}

impl BaseEvent {
    pub fn new(target: DynElement) -> Self {
        Self {
            target,
            current_target: target,
            propagation_stopped: false,
            default_prevented: false,
        }
    }

    fn retarget(&mut self, target: DynElement) {
        self.target = target;
        self.current_target = target;
    }
}

/// Behavior shared by every concrete RetGui event.
///
/// This trait must be in scope to call its methods on a concrete event.
pub trait Event {
    #[doc(hidden)]
    fn base(&self) -> &BaseEvent;

    #[doc(hidden)]
    fn base_mut(&mut self) -> &mut BaseEvent;

    /// Returns the element at which the event was originally dispatched.
    fn target(&self) -> DynElement {
        self.base().target
    }

    /// Returns the element whose handlers are currently being invoked.
    fn current_target(&self) -> DynElement {
        self.base().current_target
    }

    /// Stops the event from reaching any remaining elements.
    fn stop_propagation(&mut self) {
        self.base_mut().propagation_stopped = true;
    }

    /// Prevents RetGui's default handling for the event.
    fn prevent_default(&mut self) {
        self.base_mut().default_prevented = true;
    }

    /// Returns whether propagation has been stopped.
    fn is_propagation_stopped(&self) -> bool {
        self.base().propagation_stopped
    }

    /// Returns whether default handling has been prevented.
    fn is_default_prevented(&self) -> bool {
        self.base().default_prevented
    }
}

/// What caused a click event.
#[derive(Clone, Debug)]
pub enum ClickTrigger {
    Pointer {
        button: Option<PointerButton>,
        position: Point,
    },
    Keyboard {
        key: Key,
    },
    Accessibility,
    Programmatic,
}

#[derive(Clone)]
pub struct ClickEvent {
    base: BaseEvent,
    pub trigger: ClickTrigger,
}

impl ClickEvent {
    pub fn new(target: DynElement, trigger: ClickTrigger) -> Self {
        Self {
            base: BaseEvent::new(target),
            trigger,
        }
    }
}

#[derive(Clone)]
pub struct PointerButtonEvent {
    base: BaseEvent,
    pub button: Option<PointerButton>,
    pub position: Point,
    pub pointer: PointerInfo,
    pub state: PointerState,
}

impl PointerButtonEvent {
    pub fn new(target: DynElement, button: Option<PointerButton>, pointer: PointerInfo, state: PointerState) -> Self {
        let position = state.logical_point();

        Self {
            base: BaseEvent::new(target),
            button,
            position,
            pointer,
            state,
        }
    }
}

#[derive(Clone)]
pub struct KeyboardEvent {
    base: BaseEvent,
    pub state: ElementState,
    pub key: Key,
    pub code: Code,
    pub location: Location,
    pub modifiers: KeyboardModifiers,
    pub repeat: bool,
    pub is_composing: bool,
}

impl KeyboardEvent {
    pub fn new(target: DynElement, event: KeyEvent, modifiers: KeyboardModifiers, is_composing: bool) -> Self {
        Self {
            base: BaseEvent::new(target),
            state: event.state,
            key: event.logical_key,
            code: event.physical_key.into(),
            location: event.location,
            modifiers,
            repeat: event.repeat,
            is_composing,
        }
    }
}

/// A semantic scroll notification.
#[derive(Clone)]
pub struct ScrollEvent {
    base: BaseEvent,
}

impl ScrollEvent {
    pub fn new(target: DynElement) -> Self {
        Self {
            base: BaseEvent::new(target),
        }
    }
}

/// An element gaining focus.
#[derive(Clone)]
pub struct FocusEvent {
    base: BaseEvent,
}

impl FocusEvent {
    pub fn new(target: DynElement) -> Self {
        Self {
            base: BaseEvent::new(target),
        }
    }
}

/// An element losing focus.
#[derive(Clone)]
pub struct UnfocusEvent {
    base: BaseEvent,
}

impl UnfocusEvent {
    pub fn new(target: DynElement) -> Self {
        Self {
            base: BaseEvent::new(target),
        }
    }
}

/// A pointer entering an element.
#[derive(Clone)]
pub struct PointerEnterEvent {
    base: BaseEvent,
}

impl PointerEnterEvent {
    pub fn new(target: DynElement) -> Self {
        Self {
            base: BaseEvent::new(target),
        }
    }
}

/// A pointer leaving an element.
#[derive(Clone)]
pub struct PointerLeaveEvent {
    base: BaseEvent,
}

impl PointerLeaveEvent {
    pub fn new(target: DynElement) -> Self {
        Self {
            base: BaseEvent::new(target),
        }
    }
}

#[derive(Clone)]
pub struct PointerCaptureEvent {
    base: BaseEvent,
    pub pointer_id: PointerId,
}

impl PointerCaptureEvent {
    pub fn new(target: DynElement, pointer_id: PointerId) -> Self {
        Self {
            base: BaseEvent::new(target),
            pointer_id,
        }
    }
}

#[derive(Clone)]
pub struct PointerMovedEvent {
    base: BaseEvent,
    pub pointer: PointerInfo,
    pub current: PointerState,
    pub coalesced: Vec<PointerState>,
    pub predicted: Vec<PointerState>,
}

impl PointerMovedEvent {
    pub fn new(target: DynElement, pointer: PointerInfo, current: PointerState) -> Self {
        Self {
            base: BaseEvent::new(target),
            pointer,
            current,
            coalesced: Vec::new(),
            predicted: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct PointerScrollEvent {
    base: BaseEvent,
    pub pointer: PointerInfo,
    pub delta: ScrollDelta,
    pub state: PointerState,
}

impl PointerScrollEvent {
    pub fn new(target: DynElement, pointer: PointerInfo, delta: ScrollDelta, state: PointerState) -> Self {
        Self {
            base: BaseEvent::new(target),
            pointer,
            delta,
            state,
        }
    }
}

#[derive(Clone)]
pub struct ImeEvent {
    base: BaseEvent,
    pub ime: Ime,
}

impl ImeEvent {
    pub fn new(target: DynElement, ime: Ime) -> Self {
        Self {
            base: BaseEvent::new(target),
            ime,
        }
    }
}

#[derive(Clone)]
pub struct TextInputChangedEvent {
    base: BaseEvent,
    pub value: String,
}

impl TextInputChangedEvent {
    pub fn new(target: DynElement, value: String) -> Self {
        Self {
            base: BaseEvent::new(target),
            value,
        }
    }
}

#[derive(Clone)]
pub struct LinkClickedEvent {
    base: BaseEvent,
    pub url: String,
}

impl LinkClickedEvent {
    pub fn new(target: DynElement, url: String) -> Self {
        Self {
            base: BaseEvent::new(target),
            url,
        }
    }
}

#[derive(Clone)]
pub struct DropdownToggledEvent {
    base: BaseEvent,
    pub is_open: bool,
}

impl DropdownToggledEvent {
    pub fn new(target: DynElement, is_open: bool) -> Self {
        Self {
            base: BaseEvent::new(target),
            is_open,
        }
    }
}

#[derive(Clone)]
pub struct DropdownItemSelectedEvent {
    base: BaseEvent,
    pub index: usize,
}

impl DropdownItemSelectedEvent {
    pub fn new(target: DynElement, index: usize) -> Self {
        Self {
            base: BaseEvent::new(target),
            index,
        }
    }
}

#[derive(Clone)]
pub struct SwitchToggledEvent {
    base: BaseEvent,
    pub toggled: bool,
}

impl SwitchToggledEvent {
    pub fn new(target: DynElement, toggled: bool) -> Self {
        Self {
            base: BaseEvent::new(target),
            toggled,
        }
    }
}

#[derive(Clone)]
pub struct SliderValueChangedEvent {
    base: BaseEvent,
    pub value: f64,
}

impl SliderValueChangedEvent {
    pub fn new(target: DynElement, value: f64) -> Self {
        Self {
            base: BaseEvent::new(target),
            value,
        }
    }
}

#[derive(Clone)]
pub struct RadioValueChangedEvent {
    base: BaseEvent,
    pub value: String,
}

impl RadioValueChangedEvent {
    pub fn new(target: DynElement, value: String) -> Self {
        Self {
            base: BaseEvent::new(target),
            value,
        }
    }
}

#[derive(Clone)]
pub struct CheckboxToggledEvent {
    base: BaseEvent,
    pub label: String,
    pub status: bool,
}

impl CheckboxToggledEvent {
    pub fn new(target: DynElement, label: String, status: bool) -> Self {
        Self {
            base: BaseEvent::new(target),
            label,
            status,
        }
    }
}

/// A type-erased application-defined event.
#[derive(Clone)]
pub struct CustomEvent {
    base: BaseEvent,
    pub data: Arc<UserEventData>,
}

impl CustomEvent {
    pub fn new<T>(target: DynElement, detail: T) -> Self
    where
        T: Any + 'static,
    {
        Self {
            base: BaseEvent::new(target),
            data: Arc::new(detail),
        }
    }

    pub fn from_arc(target: DynElement, detail: Arc<UserEventData>) -> Self {
        Self {
            base: BaseEvent::new(target),
            data: detail,
        }
    }

    pub fn data<T: Any>(&self) -> Option<&T> {
        self.data.downcast_ref()
    }
}

impl Event for ClickEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for PointerButtonEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for KeyboardEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for ScrollEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for FocusEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for UnfocusEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for PointerCaptureEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for PointerEnterEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for PointerLeaveEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for PointerMovedEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for PointerScrollEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for ImeEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for TextInputChangedEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for LinkClickedEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for DropdownToggledEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for DropdownItemSelectedEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for SwitchToggledEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for SliderValueChangedEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for RadioValueChangedEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for CheckboxToggledEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

impl Event for CustomEvent {
    fn base(&self) -> &BaseEvent {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        &mut self.base
    }
}

pub type CheckboxToggledHandler<S = ()> = Rc<dyn Fn(&mut CheckboxToggledEvent, &mut App<S>, &mut S)>;
pub type ClickHandler<S = ()> = Rc<dyn Fn(&mut ClickEvent, &mut App<S>, &mut S)>;
pub type CustomHandler<S = ()> = Rc<dyn Fn(&mut CustomEvent, &mut App<S>, &mut S)>;
pub type DropdownItemSelectedHandler<S = ()> = Rc<dyn Fn(&mut DropdownItemSelectedEvent, &mut App<S>, &mut S)>;
pub type FocusHandler<S = ()> = Rc<dyn Fn(&mut FocusEvent, &mut App<S>, &mut S)>;
pub type KeyboardInputHandler<S = ()> = Rc<dyn Fn(&mut KeyboardEvent, &mut App<S>, &mut S)>;
pub type PointerCaptureHandler<S = ()> = Rc<dyn Fn(&mut PointerCaptureEvent, &mut App<S>, &mut S)>;
pub type PointerEnterHandler<S = ()> = Rc<dyn Fn(&mut PointerEnterEvent, &mut App<S>, &mut S)>;
pub type PointerEventHandler<S = ()> = Rc<dyn Fn(&mut PointerButtonEvent, &mut App<S>, &mut S)>;
pub type PointerLeaveHandler<S = ()> = Rc<dyn Fn(&mut PointerLeaveEvent, &mut App<S>, &mut S)>;
pub type PointerMovedHandler<S = ()> = Rc<dyn Fn(&mut PointerMovedEvent, &mut App<S>, &mut S)>;
pub type PointerUpdateHandler<S = ()> = PointerMovedHandler<S>;
pub type RadioValueChangedHandler<S = ()> = Rc<dyn Fn(&mut RadioValueChangedEvent, &mut App<S>, &mut S)>;
pub type ScrollHandler<S = ()> = Rc<dyn Fn(&mut ScrollEvent, &mut App<S>, &mut S)>;
pub type SliderValueChangedHandler<S = ()> = Rc<dyn Fn(&mut SliderValueChangedEvent, &mut App<S>, &mut S)>;
pub type TextInputChangedHandler<S = ()> = Rc<dyn Fn(&mut TextInputChangedEvent, &mut App<S>, &mut S)>;
pub type UnfocusHandler<S = ()> = Rc<dyn Fn(&mut UnfocusEvent, &mut App<S>, &mut S)>;
pub type UserEventData = dyn Any;

pub enum EventCallbackKind<S: 'static = ()> {
    CheckboxToggled(CheckboxToggledHandler<S>),
    Click(ClickHandler<S>),
    Custom(CustomHandler<S>),
    DropdownItemSelected(DropdownItemSelectedHandler<S>),
    Focus(FocusHandler<S>),
    GotPointerCapture(PointerCaptureHandler<S>),
    KeyboardInput(KeyboardInputHandler<S>),
    LostPointerCapture(PointerCaptureHandler<S>),
    PointerButtonDown(PointerEventHandler<S>),
    PointerButtonUp(PointerEventHandler<S>),
    PointerEnter(PointerEnterHandler<S>),
    PointerLeave(PointerLeaveHandler<S>),
    PointerMoved(PointerMovedHandler<S>),
    RadioValueChanged(RadioValueChangedHandler<S>),
    Scroll(ScrollHandler<S>),
    SliderValueChanged(SliderValueChangedHandler<S>),
    TextInputChanged(TextInputChangedHandler<S>),
    Unfocus(UnfocusHandler<S>),
}

impl<S: 'static> Clone for EventCallbackKind<S> {
    fn clone(&self) -> Self {
        match self {
            Self::CheckboxToggled(handler) => Self::CheckboxToggled(handler.clone()),
            Self::Click(handler) => Self::Click(handler.clone()),
            Self::Custom(handler) => Self::Custom(handler.clone()),
            Self::DropdownItemSelected(handler) => Self::DropdownItemSelected(handler.clone()),
            Self::Focus(handler) => Self::Focus(handler.clone()),
            Self::GotPointerCapture(handler) => Self::GotPointerCapture(handler.clone()),
            Self::KeyboardInput(handler) => Self::KeyboardInput(handler.clone()),
            Self::LostPointerCapture(handler) => Self::LostPointerCapture(handler.clone()),
            Self::PointerButtonDown(handler) => Self::PointerButtonDown(handler.clone()),
            Self::PointerButtonUp(handler) => Self::PointerButtonUp(handler.clone()),
            Self::PointerEnter(handler) => Self::PointerEnter(handler.clone()),
            Self::PointerLeave(handler) => Self::PointerLeave(handler.clone()),
            Self::PointerMoved(handler) => Self::PointerMoved(handler.clone()),
            Self::RadioValueChanged(handler) => Self::RadioValueChanged(handler.clone()),
            Self::Scroll(handler) => Self::Scroll(handler.clone()),
            Self::SliderValueChanged(handler) => Self::SliderValueChanged(handler.clone()),
            Self::TextInputChanged(handler) => Self::TextInputChanged(handler.clone()),
            Self::Unfocus(handler) => Self::Unfocus(handler.clone()),
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct EventListenerOptions {
    pub capturing: bool,
}

pub struct EventCallback<S: 'static = ()> {
    pub callback: EventCallbackKind<S>,
    pub capturing: bool,
}

impl<S: 'static> Clone for EventCallback<S> {
    fn clone(&self) -> Self {
        Self {
            callback: self.callback.clone(),
            capturing: self.capturing,
        }
    }
}

/// The inline event representation used by RetGui's dispatcher and queue.
///
/// Most users should use concrete event types through the typed callback APIs.
#[doc(hidden)]
#[derive(Clone)]
pub enum EventKind {
    GotPointerCapture(PointerCaptureEvent),
    LostPointerCapture(PointerCaptureEvent),
    PointerEnter(PointerEnterEvent),
    PointerLeave(PointerLeaveEvent),
    PointerUp(PointerButtonEvent),
    PointerDown(PointerButtonEvent),
    Click(ClickEvent),
    Focus(FocusEvent),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
    PointerMoved(PointerMovedEvent),
    PointerScroll(PointerScrollEvent),
    Scroll(ScrollEvent),
    Ime(ImeEvent),
    TextInputChanged(TextInputChangedEvent),
    LinkClicked(LinkClickedEvent),
    DropdownToggled(DropdownToggledEvent),
    DropdownItemSelected(DropdownItemSelectedEvent),
    SwitchToggled(SwitchToggledEvent),
    SliderValueChanged(SliderValueChangedEvent),
    Custom(CustomEvent),
    RadioValueChanged(RadioValueChangedEvent),
    Unfocus(UnfocusEvent),
    CheckboxToggled(CheckboxToggledEvent),
}

impl EventKind {
    pub(super) fn is_system_pointer_event(&self) -> bool {
        matches!(
            self,
            Self::PointerMoved(_) | Self::PointerUp(_) | Self::PointerDown(_) | Self::PointerScroll(_)
        )
    }

    pub(super) fn pointer_id(&self) -> Option<PointerId> {
        match self {
            Self::GotPointerCapture(event) | Self::LostPointerCapture(event) => Some(event.pointer_id),
            Self::PointerUp(event) | Self::PointerDown(event) => event.pointer.pointer_id,
            Self::PointerMoved(event) => event.pointer.pointer_id,
            Self::PointerScroll(event) => event.pointer.pointer_id,
            _ => None,
        }
    }

    pub(super) fn is_keyboard_event(&self) -> bool {
        matches!(self, Self::KeyDown(_) | Self::KeyUp(_) | Self::Ime(_))
    }

    pub(crate) fn retarget(&mut self, target: DynElement) {
        self.base_mut().retarget(target);
    }
}

impl Event for EventKind {
    fn base(&self) -> &BaseEvent {
        match self {
            Self::GotPointerCapture(event) => event.base(),
            Self::LostPointerCapture(event) => event.base(),
            Self::PointerEnter(event) => event.base(),
            Self::PointerLeave(event) => event.base(),
            Self::PointerUp(event) => event.base(),
            Self::PointerDown(event) => event.base(),
            Self::Click(event) => event.base(),
            Self::Focus(event) => event.base(),
            Self::KeyDown(event) => event.base(),
            Self::KeyUp(event) => event.base(),
            Self::PointerMoved(event) => event.base(),
            Self::PointerScroll(event) => event.base(),
            Self::Scroll(event) => event.base(),
            Self::Ime(event) => event.base(),
            Self::TextInputChanged(event) => event.base(),
            Self::LinkClicked(event) => event.base(),
            Self::DropdownToggled(event) => event.base(),
            Self::DropdownItemSelected(event) => event.base(),
            Self::SwitchToggled(event) => event.base(),
            Self::SliderValueChanged(event) => event.base(),
            Self::Custom(event) => event.base(),
            Self::RadioValueChanged(event) => event.base(),
            Self::Unfocus(event) => event.base(),
            Self::CheckboxToggled(event) => event.base(),
        }
    }

    fn base_mut(&mut self) -> &mut BaseEvent {
        match self {
            Self::GotPointerCapture(event) => event.base_mut(),
            Self::LostPointerCapture(event) => event.base_mut(),
            Self::PointerEnter(event) => event.base_mut(),
            Self::PointerLeave(event) => event.base_mut(),
            Self::PointerUp(event) => event.base_mut(),
            Self::PointerDown(event) => event.base_mut(),
            Self::Click(event) => event.base_mut(),
            Self::Focus(event) => event.base_mut(),
            Self::KeyDown(event) => event.base_mut(),
            Self::KeyUp(event) => event.base_mut(),
            Self::PointerMoved(event) => event.base_mut(),
            Self::PointerScroll(event) => event.base_mut(),
            Self::Scroll(event) => event.base_mut(),
            Self::Ime(event) => event.base_mut(),
            Self::TextInputChanged(event) => event.base_mut(),
            Self::LinkClicked(event) => event.base_mut(),
            Self::DropdownToggled(event) => event.base_mut(),
            Self::DropdownItemSelected(event) => event.base_mut(),
            Self::SwitchToggled(event) => event.base_mut(),
            Self::SliderValueChanged(event) => event.base_mut(),
            Self::Custom(event) => event.base_mut(),
            Self::RadioValueChanged(event) => event.base_mut(),
            Self::Unfocus(event) => event.base_mut(),
            Self::CheckboxToggled(event) => event.base_mut(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::event_dispatch::dispatch_event;
    use super::helpers::freeze_target_list;
    use super::{
        ClickEvent, ClickTrigger, DynElement, Event, EventCallbackKind, EventKind, EventListenerOptions, FocusEvent,
    };
    use crate::App;
    use crate::elements::{Container, Element, Text};

    fn event_target(app: &mut App) -> DynElement {
        DynElement::new(Container::new(app).inner)
    }

    fn click<S: 'static>(app: &mut App<S>, state: &mut S, target: DynElement) {
        let targets = freeze_target_list(target, &app.elements);
        let mut event = EventKind::Click(ClickEvent::new(target, ClickTrigger::Programmatic));
        dispatch_event(&mut event, &targets, app, state);
    }

    #[test]
    fn event_controls_track_dispatch_state() {
        let mut app = App::new();
        let target = event_target(&mut app);
        let mut event = FocusEvent::new(target);

        assert!(event.target() == target);
        assert!(event.current_target() == target);
        assert!(!event.is_propagation_stopped());
        assert!(!event.is_default_prevented());

        event.stop_propagation();
        event.prevent_default();

        assert!(event.is_propagation_stopped());
        assert!(event.is_default_prevented());
    }

    #[test]
    fn retarget_updates_both_targets() {
        let mut app = App::new();
        let original = event_target(&mut app);
        let replacement = event_target(&mut app);
        assert!(original != replacement);
        let mut event = EventKind::Focus(FocusEvent::new(original));

        event.retarget(replacement);

        assert!(event.target() == replacement);
        assert!(event.current_target() == replacement);
    }

    #[test]
    fn deleting_an_event_target_during_dispatch_is_safe() {
        let mut app = App::<Vec<&'static str>>::new();
        let mut state = Vec::new();
        let parent = Container::new(&mut app);
        let child = Container::new(&mut app);
        let captured = Rc::new(());
        let weak = Rc::downgrade(&captured);
        child.add_click_listener(&mut app, move |event, app, state| {
            parent.delete_all_children(app);
            assert_eq!(Rc::strong_count(&captured), 1);
            state.push("deleted");
            event.stop_propagation();
        });
        child.add_click_listener(&mut app, |_event, _app, state| state.push("remaining"));
        parent.add_click_listener(&mut app, |_event, _app, state| state.push("parent"));
        parent.push(&mut app, child);

        click(&mut app, &mut state, child.inner);

        assert!(!app.contains(child.inner));
        assert_eq!(state, ["deleted", "remaining"]);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn cloned_subtrees_keep_listener_order_and_independent_lists() {
        let mut app = App::<Vec<(&'static str, DynElement)>>::new();
        let mut state = Vec::new();
        let parent = Container::new(&mut app);
        let child = Container::new(&mut app);
        parent.push(&mut app, child);
        parent.add_event_listener(
            &mut app,
            EventCallbackKind::Click(Rc::new(|event, _app, state| {
                state.push(("capture", event.current_target()));
            })),
            EventListenerOptions { capturing: true },
        );
        parent.add_click_listener(&mut app, |event, _app, state| {
            state.push(("bubble", event.current_target()));
        });
        child.add_click_listener(&mut app, |event, _app, state| {
            state.push(("child", event.current_target()));
        });

        let cloned = app.deep_clone(parent.inner);
        let cloned_child = cloned.first_child(&app).unwrap();
        child.add_click_listener(&mut app, |event, _app, state| {
            state.push(("original", event.current_target()));
        });
        cloned_child.add_click_listener(&mut app, |event, _app, state| {
            state.push(("clone", event.current_target()));
        });

        click(&mut app, &mut state, cloned_child);
        click(&mut app, &mut state, child.inner);

        assert_eq!(
            state,
            [
                ("capture", cloned),
                ("child", cloned_child),
                ("clone", cloned_child),
                ("bubble", cloned),
                ("capture", parent.inner),
                ("child", child.inner),
                ("original", child.inner),
                ("bubble", parent.inner),
            ]
        );
    }

    #[test]
    fn the_same_callback_can_dispatch_again_with_non_clone_user_state() {
        struct State {
            count: usize,
            label: Text,
            calls: Vec<(usize, &'static str)>,
        }

        impl State {
            fn increment(&mut self, app: &mut App<Self>) {
                self.count += 1;
                self.refresh(app);
            }

            fn refresh(&self, app: &mut App<Self>) {
                self.label.set_text(app, &self.count.to_string());
            }
        }

        let mut app = App::<State>::new();
        let target = Container::new(&mut app);
        let label = Text::new(&mut app, "0");
        let mut state = State {
            count: 0,
            label,
            calls: Vec::new(),
        };
        target.add_click_listener(&mut app, |event, app, state| {
            state.increment(app);
            let count = state.count;
            state.calls.push((count, "enter"));
            if count == 1 {
                click(app, state, event.current_target());
            }
            state.calls.push((count, "leave"));
        });

        let cloned = app.deep_clone(target.inner);
        click(&mut app, &mut state, cloned);

        assert_eq!(state.count, 2);
        assert_eq!(state.label.text(&app), "2");
        assert_eq!(state.calls, [(1, "enter"), (2, "enter"), (2, "leave"), (1, "leave")]);
    }

    #[test]
    fn listeners_added_during_capture_join_later_snapshots() {
        let mut app = App::<Vec<&'static str>>::new();
        let mut state = Vec::new();
        let parent = Container::new(&mut app);
        let child = Container::new(&mut app);
        parent.push(&mut app, child);
        parent.add_event_listener(
            &mut app,
            EventCallbackKind::Click(Rc::new(|_event, _app, state| state.push("parent capture"))),
            EventListenerOptions { capturing: true },
        );
        child.add_event_listener(
            &mut app,
            EventCallbackKind::Click(Rc::new(move |_event, app, state| {
                state.push("capture");
                child.add_click_listener(app, |_event, _app, state| state.push("added bubble"));
                child.add_event_listener(
                    app,
                    EventCallbackKind::Click(Rc::new(|_event, _app, state| state.push("added capture"))),
                    EventListenerOptions { capturing: true },
                );
            })),
            EventListenerOptions { capturing: true },
        );
        child.add_click_listener(&mut app, |_event, _app, state| state.push("bubble"));
        parent.add_click_listener(&mut app, |_event, _app, state| state.push("parent bubble"));

        click(&mut app, &mut state, child.inner);
        assert_eq!(
            state,
            ["parent capture", "capture", "bubble", "added bubble", "parent bubble"]
        );

        state.clear();
        click(&mut app, &mut state, child.inner);
        assert_eq!(
            state,
            [
                "parent capture",
                "capture",
                "added capture",
                "bubble",
                "added bubble",
                "added bubble",
                "parent bubble"
            ]
        );
    }

    #[test]
    fn detached_callbacks_live_until_their_subtree_is_deleted() {
        let mut app = App::<usize>::new();
        let mut state = 0;
        let parent = Container::new(&mut app);
        let subtree = Container::new(&mut app);
        let child = Container::new(&mut app);
        let captured = Rc::new(1);
        let weak = Rc::downgrade(&captured);
        child.add_click_listener(&mut app, move |_event, _app, count| *count += *captured);
        parent.push(&mut app, subtree);
        subtree.push(&mut app, child);

        parent.remove_all_children(&mut app);
        parent.delete_all_children(&mut app);
        click(&mut app, &mut state, child.inner);
        assert_eq!(state, 1);
        assert!(weak.upgrade().is_some());

        parent.push(&mut app, subtree);
        parent.delete_all_children(&mut app);
        assert!(weak.upgrade().is_none());
        assert!(!app.contains(subtree.inner));
        assert!(!app.contains(child.inner));
    }
}
