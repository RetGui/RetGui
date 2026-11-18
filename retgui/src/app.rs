use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::{Rc, Weak};
use std::sync::Arc;
#[cfg(all(feature = "audio", not(target_arch = "wasm32")))]
use std::time::{Duration, Instant};

use gummy::NodeId;

use issho::AccessEvent;

use retgui_logging::info;

use retgui_primitives::geometry::Size;

use retgui_resource_manager::resource_type::ResourceType;
use retgui_resource_manager::{ResourceId, ResourceManager};

use retgui_runtime::RetGuiRuntime;

#[cfg(all(feature = "audio", target_arch = "wasm32"))]
use web_time::{Duration, Instant};

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, Ime, KeyEvent, MouseScrollDelta, PointerKind, WindowEvent};
use winit::event_loop::ActiveEventLoop;

use crate::RetGuiOptions;
use crate::accessibility::ACCESS_TREE;
#[cfg(feature = "audio")]
use crate::elements::{AUDIO_CONTEXT, AudioInner};
use crate::elements::{DynElement, ElementIdMap, ElementInternals, Window, scrollable, set_focus_outline_visible};
use crate::events::{EventDispatcher, EventKind, ImeEvent, KeyboardEvent, PointerButtonEvent, PointerInfo, PointerMovedEvent, PointerScrollEvent, PointerState};
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;
use crate::window_manager::WindowManager;

thread_local! {
    pub(crate) static ELEMENTS: RefCell<ElementIdMap> = RefCell::new(ElementIdMap::new());
    pub(crate) static PENDING_RESOURCES: RefCell<VecDeque<(ResourceId, ResourceType)>> = const { RefCell::new(VecDeque::new()) };
    pub(crate) static IN_PROGRESS_RESOURCES: RefCell<VecDeque<(ResourceId, ResourceType)>> = const { RefCell::new(VecDeque::new()) };
    pub(crate) static FOCUS: RefCell<Option<Weak<RefCell<dyn ElementInternals>>>> = RefCell::new(None);
    pub(crate) static WINDOW_MANAGER: RefCell<WindowManager> = RefCell::new(WindowManager::new());
    pub(crate) static GUMMY_TREE: RefCell<GummyTree> = RefCell::new(GummyTree::new());
    /// An event queue that users or elements can manipulate. Cleared at the start and end of every event dispatch.
    static EVENT_DISPATCH_QUEUE: RefCell<VecDeque<EventKind>> = RefCell::new(VecDeque::with_capacity(10));
}

/// Update interval for audio elements.
#[cfg(feature = "audio")]
const AUDIO_UI_UPDATE_INTERVAL: Duration = Duration::from_millis(250);

#[cfg(feature = "audio")]
fn audio_ui_update_due(last_update: Option<Instant>, now: Instant) -> bool {
    last_update.is_none_or(|last_update| now.duration_since(last_update) >= AUDIO_UI_UPDATE_INTERVAL)
}

pub struct App {
    pub(crate) clipboard: crate::clipboard::Clipboard,
    pub(crate) event_dispatcher: Rc<RefCell<EventDispatcher>>,
    /// The text context is used to manage fonts and text rendering. It is only valid between resume and pause.
    pub(crate) text_context: Rc<RefCell<Option<TextContext>>>,
    pub(crate) reload_fonts: bool,
    /// The resource manager is used to manage resources such as images and fonts.
    ///
    /// The resource manager is responsible for loading, caching, and providing access to resources.
    pub(crate) resource_manager: Arc<ResourceManager>,

    pub(crate) runtime: RetGuiRuntime,

    #[allow(dead_code)]
    pub(crate) retgui_options: RetGuiOptions,

    /// True if the winit app is active.
    pub(crate) active: bool,
    pub(crate) wait_cancelled: bool,
    pub(crate) close_requested: bool,

    #[cfg(feature = "audio")]
    pub(crate) last_audio_ui_update: Option<Instant>,
}

impl App {
    pub(crate) fn new(retgui_options: RetGuiOptions) -> Self {
        let runtime = RetGuiRuntime::new();
        info!("Created async runtime");

        #[allow(clippy::arc_with_non_send_sync)]
        let resource_manager = Arc::new(ResourceManager::new(runtime.handle()));

        let (event_dispatcher, text_context) =
            ACCESS_TREE.with(|access_tree| (access_tree.event_dispatcher.clone(), access_tree.text_context.clone()));

        Self {
            event_dispatcher,
            text_context,
            resource_manager,
            reload_fonts: false,
            clipboard: Default::default(),
            runtime,
            retgui_options,
            active: false,
            wait_cancelled: false,
            close_requested: false,
            #[cfg(feature = "audio")]
            last_audio_ui_update: None,
        }
    }

    /// Handle window events.
    pub fn on_window_event(&mut self, window: Window, event: WindowEvent) {
        if let Some(context) = self.text_context.borrow_mut().as_mut() {
            self.clipboard.receive(&event, context);
        }
        match event {
            WindowEvent::KeyboardInput { event, .. } => self.on_keyboard_input(window, event),
            WindowEvent::ModifiersChanged(modifiers) => {
                window.inner.borrow_mut().update_modifiers(modifiers.state());
            }
            WindowEvent::PointerMoved {
                position,
                primary,
                source,
                ..
            } => {
                let scale_factor = window.effective_scale_factor();
                let pointer = PointerInfo::from_source(&source, primary);
                self.on_pointer_moved(window, pointer, PointerState::new(position, scale_factor));
            }
            WindowEvent::PointerButton {
                state,
                position,
                primary,
                button,
                ..
            } => {
                let scale_factor = window.effective_scale_factor();
                let pointer = PointerInfo::from_button(&button, primary);
                self.on_pointer_button(
                    window,
                    button.mouse_button(),
                    pointer,
                    PointerState::new(position, scale_factor),
                    state == ElementState::Released,
                );
            }
            WindowEvent::MouseWheel { delta, .. } => self.on_mouse_wheel(window, delta),
            WindowEvent::CloseRequested => self.close_window(&window),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.on_scale_factor_changed(window, scale_factor);
            }
            WindowEvent::SurfaceResized(new_size) => {
                self.on_resize(window, Size::new(new_size.width as f32, new_size.height as f32));
            }
            WindowEvent::Ime(ime) => self.on_ime(window, ime),
            WindowEvent::RedrawRequested => self.on_redraw_requested(window),
            WindowEvent::Moved(_) => self.on_move(window),
            WindowEvent::Focused(focused) => {
                if !focused {
                    window.inner.borrow_mut().ime_composing = false;
                }
                window.on_focused(focused);
            }
            _ => {}
        }
    }

    fn on_mouse_wheel(&mut self, window: Window, delta: MouseScrollDelta) {
        let scale_factor = window.effective_scale_factor();
        let logical_position = window.mouse_position().unwrap_or_default();
        let physical_position =
            PhysicalPosition::new(logical_position.x * scale_factor, logical_position.y * scale_factor);
        self.on_pointer_scroll(
            window,
            PointerInfo::new(PointerKind::Mouse, true),
            delta,
            PointerState::new(physical_position, scale_factor),
        );
    }

    fn close_window(&mut self, window: &Window) {
        WINDOW_MANAGER.with_borrow_mut(|window_manager| {
            window_manager.close_window(window, &self.runtime);
            if window_manager.is_empty() {
                self.on_close_requested();
                self.close_requested = true;
            }
        });
    }

    pub fn on_close_requested(&mut self) {
        info!("RetGui application is closing.");
    }

    pub fn on_scale_factor_changed(&mut self, window: Window, scale_factor: f64) {
        window.on_scale_factor_changed(scale_factor);
    }

    pub fn on_resume(&mut self, event_loop: Option<&dyn ActiveEventLoop>) {
        self.active = true;
        self.setup_text_context();

        WINDOW_MANAGER.with_borrow_mut(|window_manager| {
            window_manager.on_resume(self, event_loop);
        });
    }

    pub(crate) fn on_render_ready(&mut self) {
        // Drain every window because native proxy wake-ups may be coalesced.
        // This also runs while suspended, so a worker cannot be left waiting
        // for presentation after the application's active state changes.
        WINDOW_MANAGER.with_borrow(|manager| {
            for window in manager.windows() {
                window.inner.borrow_mut().poll_completed_frames();
            }
        });
    }

    pub fn on_about_to_wait(&mut self, event_loop: Option<&dyn ActiveEventLoop>) {
        if let Some(context) = self.text_context.borrow_mut().as_mut() {
            self.clipboard.process(event_loop, context);
        }
        self.poll_runtime();
        self.process_resources();
        if let Some(text_context) = self.text_context.borrow_mut().as_mut() {
            self.event_dispatcher.borrow_mut().dispatch_queued_events(text_context);
        }

        #[cfg(feature = "audio")]
        self.update_audio_ui();

        WINDOW_MANAGER.with_borrow_mut(|window_manager| {
            window_manager.on_about_to_wait(self, event_loop);
        });
    }

    fn poll_runtime(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        self.runtime.maybe_block_on(async {
            retgui_runtime::task::yield_now().await;
        });
        self.runtime.handle().update_local_set();
    }

    fn process_resources(&mut self) {
        let mut resource_loaded = false;
        IN_PROGRESS_RESOURCES.with_borrow_mut(|in_progress| {
            in_progress.retain(|(resource_id, resource_type)| {
                if !self.resource_manager.contains(resource_id) {
                    return true;
                }

                resource_loaded = true;
                if *resource_type == ResourceType::Font {
                    self.reload_fonts = true;
                }
                false
            });
        });

        if resource_loaded {
            WINDOW_MANAGER.with_borrow_mut(|window_manager| {
                window_manager.dirty_and_redraw_all_windows(self);
            });
        }
    }

    #[cfg(feature = "audio")]
    fn update_audio_ui(&mut self) {
        let now = Instant::now();
        if !audio_ui_update_due(self.last_audio_ui_update, now) {
            return;
        }
        self.last_audio_ui_update = Some(now);

        AUDIO_CONTEXT.with(|audio_context| {
            if let Some(ctx) = audio_context.get() {
                for audio_element in &ctx.borrow().sounds {
                    update_audio_element(*audio_element);
                }
            }
        });
    }

    /// Suspends rendering until the application is resumed again.
    pub fn on_suspended(&mut self) {
        self.active = false;
    }

    /// Returns the window associated with a winit window ID.
    pub fn window_by_id(&self, window_id: winit::window::WindowId) -> Option<Window> {
        WINDOW_MANAGER.with_borrow(|window_manager| window_manager.get_window_by_id(window_id))
    }

    /// Returns whether the driver should exit its event loop.
    pub fn close_requested(&self) -> bool {
        self.close_requested
    }

    /// Handles the window resize event.
    pub fn on_resize(&mut self, window: Window, new_size: Size<f32>) {
        window.on_resize(new_size);
        if new_size.width <= 0.0 || new_size.height <= 0.0 || !self.active {
            return;
        }
        window.wait_for_render();
        self.on_redraw_requested(window.clone());
        window.wait_for_render();
    }

    /// Updates the tree, layouts the elements, and draws the view.
    pub fn on_redraw_requested(&mut self, window: Window) {
        let delta = window.animation_delta();
        let has_active_animations =
            WINDOW_MANAGER.with_borrow_mut(|window_manager| window_manager.animation_tick(&window, delta));
        self.update_resources();
        window.on_redraw_requested(self);

        if has_active_animations && window.winit_window().is_some() {
            window.request_redraw();
        }
    }

    pub fn on_move(&mut self, _window: Window) {}

    pub fn on_pointer_scroll(
        &mut self,
        window: Window,
        pointer: PointerInfo,
        delta: MouseScrollDelta,
        state: PointerState,
    ) {
        let pointer_scroll_update =
            PointerScrollEvent::new(DynElement::new(window.inner.clone()), pointer, delta, state);
        if window.inner.borrow_mut().maybe_zoom(&pointer_scroll_update) {
            return;
        }
        self.dispatch_event(&window, EventKind::PointerScroll(pointer_scroll_update));
    }

    pub fn on_pointer_button(
        &mut self,
        window: Window,
        button: Option<winit::event::MouseButton>,
        pointer: PointerInfo,
        state: PointerState,
        is_up: bool,
    ) {
        if !is_up {
            set_focus_outline_visible(false);
        }

        let cursor_position = state.logical_point();
        let pointer_event = PointerButtonEvent::new(DynElement::new(window.inner.clone()), button, pointer, state);

        let event = if is_up {
            EventKind::PointerUp(pointer_event)
        } else {
            EventKind::PointerDown(pointer_event)
        };
        window.set_mouse_position(Some(cursor_position));

        self.dispatch_event(&window, event);
    }

    pub fn on_pointer_moved(&mut self, window: Window, pointer: PointerInfo, state: PointerState) {
        window.set_mouse_position(Some(state.logical_point()));
        let event = PointerMovedEvent::new(DynElement::new(window.inner.clone()), pointer, state);
        self.dispatch_event(&window, EventKind::PointerMoved(event));
    }

    pub fn on_ime(&mut self, window: Window, ime: Ime) {
        if let Some(is_composing) = match &ime {
            Ime::Preedit(text, _) => Some(!text.is_empty()),
            Ime::Enabled | Ime::Commit(_) | Ime::Disabled => Some(false),
            _ => None,
        } {
            window.inner.borrow_mut().ime_composing = is_composing;
        }
        let event = ImeEvent::new(DynElement::new(window.inner.clone()), ime);
        self.dispatch_event(&window, EventKind::Ime(event));
    }

    pub fn on_keyboard_input(&mut self, window: Window, keyboard_input: KeyEvent) {
        let (modifiers, is_composing) = {
            let window = window.inner.borrow();
            (window.modifiers, window.ime_composing)
        };
        let state = keyboard_input.state;
        if state == ElementState::Pressed && !modifiers.control_key() && !modifiers.alt_key() && !modifiers.meta_key() {
            set_focus_outline_visible(true);
        }
        let event = KeyboardEvent::new(
            DynElement::new(window.inner.clone()),
            keyboard_input,
            modifiers,
            is_composing,
        );
        if window.inner.borrow_mut().maybe_toggle_perf_stats(&event) {
            return;
        }
        if window.inner.borrow_mut().maybe_zoom_keyboard(&event) {
            return;
        }
        let navigation_target = window.inner.borrow().tab_navigation_target(&event);
        let event = match state {
            ElementState::Pressed => EventKind::KeyDown(event),
            ElementState::Released => EventKind::KeyUp(event),
        };
        let prevent_defaults = self.dispatch_event(&window, event);
        if !prevent_defaults && let Some(target) = navigation_target {
            target.borrow_mut().focus();
            scrollable::handle_accessibility_scroll_event(&mut *target.borrow_mut(), &AccessEvent::ScrollIntoView);
            window.request_redraw();
        }
    }

    fn dispatch_event(&mut self, window: &Window, mut event: EventKind) -> bool {
        let mouse_pos = window.mouse_position();
        self.event_dispatcher.borrow_mut().dispatch_event(
            &mut event,
            mouse_pos,
            window.inner.clone(),
            self.text_context.borrow_mut().as_mut().unwrap(),
        )
    }

    fn update_resources(&mut self) {
        PENDING_RESOURCES.with_borrow_mut(|pending_resources| {
            IN_PROGRESS_RESOURCES.with_borrow_mut(|in_progress| {
                for (resource, resource_type) in pending_resources.drain(..) {
                    if self.resource_manager.contains(&resource)
                        || in_progress.contains(&(resource.clone(), resource_type.clone()))
                    {
                        continue;
                    }
                    self.resource_manager
                        .async_download_resource(resource.clone(), &resource_type);
                    in_progress.push_back((resource, resource_type));
                }
            });
        });
    }

    /// Initialize any data needed to layout/render text.
    fn setup_text_context(&mut self) {
        self.text_context.borrow_mut().get_or_insert_with(|| {
            #[cfg(all(not(target_arch = "wasm32"), feature = "system_fonts"))]
            let context = TextContext::new();
            #[cfg(any(target_arch = "wasm32", not(feature = "system_fonts")))]
            let context = {
                let mut context = TextContext::new();
                register_fallback_fonts(&mut context);
                context
            };
            context
        });
    }
}

#[cfg(feature = "audio")]
fn update_audio_element(id: u64) {
    use std::any::Any;

    let Some(element) = ELEMENTS.with_borrow(|elements| elements.get(id).cloned()) else {
        return;
    };
    let element = element.upgrade().unwrap();
    let mut element = element.borrow_mut();
    let audio = (&mut *element as &mut dyn Any)
        .downcast_mut::<AudioInner>()
        .expect("Failed to downcast");
    audio.update();
}

#[cfg(any(target_arch = "wasm32", not(feature = "system_fonts")))]
fn register_fallback_fonts(context: &mut TextContext) {
    let fonts: [&'static [u8]; 2] = [
        include_bytes!("../../assets/fonts/Roboto-VariableFont_wdth,wght.ttf"),
        include_bytes!("../../assets/fonts/Roboto-Italic-VariableFont_wdth,wght.ttf"),
    ];
    let collection = &mut context.font_context.collection;
    for data in fonts {
        let blob = peniko::Blob::new(Arc::new(data));
        let families = collection.register_fonts(blob, None);
        collection.append_generic_families(parley::GenericFamily::SystemUi, families.iter().map(|family| family.0));
    }
}

#[cfg(all(test, feature = "audio"))]
mod tests {
    use super::{AUDIO_UI_UPDATE_INTERVAL, Duration, Instant, audio_ui_update_due};

    #[test]
    fn audio_ui_updates_are_throttled() {
        let first_update = Instant::now();

        assert!(audio_ui_update_due(None, first_update));
        assert!(!audio_ui_update_due(
            Some(first_update),
            first_update + AUDIO_UI_UPDATE_INTERVAL - Duration::from_millis(1),
        ));
        assert!(audio_ui_update_due(
            Some(first_update),
            first_update + AUDIO_UI_UPDATE_INTERVAL,
        ));
    }
}

/// Enqueues an event at the back of the dispatch queue.
pub fn queue_event(event: EventKind) {
    EVENT_DISPATCH_QUEUE.with_borrow_mut(|event_queue| {
        event_queue.push_back(event);
    });
}

/// Pops from the front of the event dispatch queue and returns the result.
pub fn dequeue_event() -> Option<EventKind> {
    EVENT_DISPATCH_QUEUE.with_borrow_mut(|event_queue| event_queue.pop_front())
}

#[inline]
pub fn request_layout(gummy_node: NodeId) {
    GUMMY_TREE.with_borrow_mut(|gummy_tree| {
        gummy_tree.mark_dirty(gummy_node);
    });
}

#[inline]
pub fn request_apply_layout(node: NodeId) {
    GUMMY_TREE.with_borrow_mut(|gummy_tree| {
        gummy_tree.request_apply_layout(node);
    });
}
