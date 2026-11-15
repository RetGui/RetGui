//! Stores one or more elements.

use retgui_logging::info;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(not(target_arch = "wasm32"))]
use std::thread;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

use gummy::AvailableSpace;

use retgui_primitives::geometry::{Point, Rectangle, Size};

use retgui_renderer::renderer::{Renderer, Screenshot};
use retgui_renderer::{RenderList, RendererType, TargetItem};

use retgui_resource_manager::ResourceManager;

#[cfg(target_arch = "wasm32")]
use {wasm_bindgen::JsCast, winit::platform::web::WindowAttributesWeb};

#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use crate::Color;
use crate::accessibility::RetGuiAccessTree;
use crate::app::{App, GUMMY_TREE, WINDOW_MANAGER};
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::{AsElement, DynElement, Element, ElementInternals, scrollable};
use crate::events::pointer_capture::PointerCapture;
use crate::events::{EventKind, KeyboardEvent, PointerScrollEvent, PointerType};
use crate::layout::GummyTree;
use crate::perf_stats::{FrameStats, LayoutStats, PerfStats, RenderStats};
#[cfg(not(target_arch = "wasm32"))]
use crate::render_message::RenderMessage;
use crate::text::text_context::TextContext;

#[cfg(not(target_arch = "wasm32"))]
use retgui_runtime::runtime::LocalRuntime;
use retgui_runtime::sync::mpsc::{UnboundedReceiver, unbounded_channel};
#[cfg(not(target_arch = "wasm32"))]
use retgui_runtime::sync::mpsc::UnboundedSender;
use retgui_runtime::RetGuiRuntime;
#[cfg(not(target_arch = "wasm32"))]
use retgui_runtime::{Receiver, Sender};

use winit::event::MouseScrollDelta;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window as WinitWindow, WindowAttributes};

pub type WindowConstructor = Box<dyn FnMut(&dyn ActiveEventLoop) -> Box<dyn WinitWindow>>;

#[derive(Clone)]
pub struct Window {
    pub(crate) inner: Rc<RefCell<WindowInternal>>,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
type TargetScratch = Rc<RefCell<Vec<Rc<RefCell<dyn ElementInternals>>>>>;

pub(crate) struct WindowInternal {
    /// The physical window size from winit.
    pub(crate) window_size: Size<f32>,
    #[cfg(target_arch = "wasm32")]
    pub(crate) renderer: Option<Box<dyn Renderer + Send>>,
    #[cfg(target_arch = "wasm32")]
    render_list: RenderList,

    // Will be empty when paused.
    pub(crate) winit_window: Option<Arc<dyn WinitWindow>>,
    pub(crate) headless: bool,
    redraw_requested: Cell<bool>,
    renderer_redraw_requested: Arc<AtomicBool>,
    pub(crate) render_buffers: Arc<crate::render_message::RenderBuffers>,

    pub(crate) access_tree: RetGuiAccessTree,
    pub(crate) pointer_capture: Rc<RefCell<PointerCapture>>,

    advanced_window_fn: Option<WindowConstructor>,
    title: Option<String>,
    /// The type of renderer to use.
    ///
    /// The renderer is chosen based on the features enabled at compile time.
    /// See [`RendererType`] for details.
    renderer_type: RendererType,
    /// The window's scale factor from winit.
    scale_factor: f64,
    /// Zoom scale factor.
    zoom_scale_factor: f64,
    perf_stats: PerfStats,
    mouse_positon: Option<Point>,
    element_data: ElementData,
    pub(crate) modifiers: ModifiersState,
    pub(crate) ime_composing: bool,

    /// The last time a frame was drawn.
    last_frame_time: Instant,

    #[cfg(not(target_arch = "wasm32"))]
    sender: Sender<RenderMessage>,
    #[cfg(not(target_arch = "wasm32"))]
    render_runtime: Option<retgui_runtime::RetGuiRuntimeHandle>,
    completed_frames: Option<UnboundedReceiver<FrameStats>>,

    pub(crate) hit_test_targets: Rc<RefCell<Vec<TargetItem>>>,
    pub(crate) target_scratch: TargetScratch,
}

impl Clone for WindowInternal {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new("RetGui")
    }
}

impl Element for Window {}

impl Drop for WindowInternal {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for Window {
    fn with<R>(&self, callback: impl FnOnce(&dyn ElementInternals) -> R) -> R {
        callback(&*self.inner.borrow())
    }

    fn with_mut<R>(&self, callback: impl FnOnce(&mut dyn ElementInternals) -> R) -> R {
        callback(&mut *self.inner.borrow_mut())
    }
}

impl crate::elements::ElementData for WindowInternal {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for WindowInternal {
    fn request_window_redraw(&self) {
        self.request_redraw();
    }

    fn pointer_capture(&self) -> Option<Rc<RefCell<PointerCapture>>> {
        Some(self.pointer_capture.clone())
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        _text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(self, gummy_tree, z_index, scale_factor);
    }

    fn draw(
        &mut self,
        render_list: &mut RenderList,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(self, render_list, resource_manager, text_context, scale_factor);
    }

    fn on_event(&mut self, event: &mut EventKind, _text_context: &mut TextContext) {
        scrollable::handle_scroll_logic(self, event);
    }

    fn push(&mut self, child: DynElement) {
        push_child_to_element(self, child.inner);
    }

    fn deep_clone(&self) -> DynElement {
        todo!()
    }
}

impl Window {
    pub fn new_advanced<F>(window_fn: F, renderer_type: RendererType) -> Self
    where
        F: FnMut(&dyn ActiveEventLoop) -> Box<dyn WinitWindow> + 'static,
    {
        let inner = WindowInternal::new(Some(window_fn), None, renderer_type);

        Window { inner }
    }

    pub fn new(title: &str) -> Self {
        let inner = WindowInternal::new(
            None::<fn(&dyn ActiveEventLoop) -> Box<dyn WinitWindow>>,
            Some(title),
            RendererType::default(),
        );

        Window { inner }
    }

    pub fn new_with_renderer(title: &str, renderer_type: RendererType) -> Self {
        let inner = WindowInternal::new(
            None::<fn(&dyn ActiveEventLoop) -> Box<dyn WinitWindow>>,
            Some(title),
            renderer_type,
        );

        Window { inner }
    }

    pub fn screenshot(&self) -> Screenshot {
        #[cfg(target_arch = "wasm32")]
        {
            self.inner.borrow_mut().screenshot()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.wait_for_render();
            let (runtime, sender) = {
                let inner = self.inner.borrow();
                (inner.render_runtime.clone(), inner.sender.clone())
            };
            let runtime = runtime.expect("window must be created before taking a screenshot");
            runtime.block_on_local(async move {
                let (reply, result) = retgui_runtime::sync::oneshot::channel();
                sender.send(RenderMessage::Screenshot(reply)).await.expect("render worker stopped");
                result.await.expect("render worker did not return a screenshot")
            })
        }
    }

    pub fn close(&self) {
        //self.inner.borrow_mut().close();
    }

    pub(crate) fn wait_for_render(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (runtime, buffers, mut frames) = {
                let mut inner = self.inner.borrow_mut();
                let Some(runtime) = inner.render_runtime.clone() else {
                    return;
                };
                (runtime, inner.render_buffers.clone(), inner.completed_frames.take().unwrap())
            };
            let frames = runtime.block_on_local(async {
                while !buffers.is_idle().await {
                    let frame = frames.recv().await.expect("render worker stopped before completing the frame");
                    self.inner.borrow_mut().perf_stats.update_stats(frame.total, frame.layout, frame.render);
                }
                frames
            });
            self.inner.borrow_mut().completed_frames = Some(frames);
        }
    }

    pub fn winit_window(&self) -> Option<Arc<dyn winit::window::Window>> {
        self.inner.borrow().winit_window()
    }

    pub fn set_winit_window(&self, window: Option<Arc<dyn WinitWindow>>) {
        self.inner.borrow_mut().set_winit_window(window)
    }

    pub fn set_scale_factor(&self, scale_factor: f64) {
        self.inner.borrow_mut().set_scale_factor(scale_factor);
    }

    /// Get the effective scale factor factoring window scale factor and zoom.
    pub fn effective_scale_factor(&self) -> f64 {
        self.inner.borrow().effective_scale_factor()
    }

    /// Get the logical size of the window.
    pub fn window_size(&self) -> Size<f32> {
        self.inner.borrow().window_size()
    }

    pub fn zoom_scale_factor(&self) -> f64 {
        self.inner.borrow().zoom_scale_factor()
    }

    pub fn on_redraw_requested(&self, retgui_app: &mut App) {
        self.inner.borrow_mut().on_redraw_requested(
            retgui_app.text_context.borrow_mut().as_mut().unwrap(),
            retgui_app.resource_manager.clone(),
        );
    }

    pub(crate) fn request_redraw(&self) {
        self.inner.borrow().request_redraw();
    }

    pub(crate) fn redraw_requested(&self) -> bool {
        self.inner.borrow().redraw_requested()
    }

    pub(crate) fn animation_delta(&self) -> Duration {
        let now = Instant::now();
        let mut inner = self.inner.borrow_mut();
        let delta = now - inner.last_frame_time;
        inner.last_frame_time = now;
        delta
    }

    pub(crate) fn reset_animation_clock(&self) {
        self.inner.borrow_mut().last_frame_time = Instant::now();
    }

    pub fn zoom_in(&self) {
        self.inner.borrow_mut().zoom_in()
    }

    pub fn zoom_out(&self) {
        self.inner.borrow_mut().zoom_out()
    }

    pub(crate) fn mouse_position(&self) -> Option<Point> {
        self.inner.borrow().mouse_position()
    }

    pub(crate) fn on_resize(&self, new_size: Size<f32>) {
        self.inner.borrow_mut().on_resize(new_size)
    }

    pub(crate) fn set_mouse_position(&self, point: Option<Point>) {
        self.inner.borrow_mut().set_mouse_position(point)
    }

    pub(crate) fn create(&self, retgui_app: &mut App, event_loop: Option<&dyn ActiveEventLoop>) {
        self.inner
            .borrow_mut()
            .create(retgui_app, event_loop, Rc::downgrade(&self.inner));
    }

    pub(crate) fn on_scale_factor_changed(&self, scale_factor: f64) {
        self.inner.borrow_mut().on_scale_factor_changed(scale_factor);
    }

    pub(crate) fn on_focused(&self, focused: bool) {
        self.inner.borrow().on_focused(focused);
    }
}

impl WindowInternal {
    pub fn new<F>(f: Option<F>, title: Option<&str>, renderer_type: RendererType) -> Rc<RefCell<Self>>
    where
        F: FnMut(&dyn ActiveEventLoop) -> Box<dyn WinitWindow> + 'static,
    {
        #[cfg(not(target_arch = "wasm32"))]
        let (sender, receiver) = retgui_runtime::sync::mpsc::channel::<RenderMessage>(1);
        let (completed_sender, completed_frames) = unbounded_channel();

        let access_tree = crate::accessibility::access_tree();
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<Self>>| {
            RefCell::new(Self {
                element_data: ElementData::new(me.clone(), true),
                window_size: Default::default(),
                scale_factor: 1.0,
                zoom_scale_factor: 1.0,
                perf_stats: PerfStats::new(),
                mouse_positon: None,
                #[cfg(target_arch = "wasm32")]
                renderer: None,
                #[cfg(target_arch = "wasm32")]
                render_list: RenderList::new(),
                winit_window: None,
                headless: false,
                redraw_requested: Cell::new(false),
                renderer_redraw_requested: Arc::new(AtomicBool::new(false)),
                render_buffers: Arc::new(crate::render_message::RenderBuffers::new()),
                access_tree: access_tree.clone(),
                advanced_window_fn: f.map(|f| Box::new(f) as WindowConstructor),
                title: title.map(|title| title.to_string()),
                renderer_type,
                pointer_capture: Default::default(),
                modifiers: Default::default(),
                ime_composing: false,
                last_frame_time: Instant::now(),
                #[cfg(not(target_arch = "wasm32"))]
                sender,
                #[cfg(not(target_arch = "wasm32"))]
                render_runtime: None,
                completed_frames: Some(completed_frames),
                hit_test_targets: Rc::new(RefCell::new(Vec::new())),
                target_scratch: Rc::new(RefCell::new(Vec::new())),
            })
        });

        inner.borrow_mut().element_data.create_layout_node(None);

        let me = Rc::downgrade(&inner);
        inner.borrow_mut().element_data.window = Some(me);

        {
            let mut inner_mut = inner.borrow_mut();
            inner_mut.element_data.set_accessibility_role(issho::Role::Window);
            if let Some(title) = inner_mut.title.clone() {
                inner_mut.element_data.set_accessibility_name(title);
            }
        }

        WINDOW_MANAGER.with_borrow_mut(|window_manager| {
            window_manager.add_window(Window {
                inner: inner.clone(),
            });
        });

        #[cfg(not(target_arch = "wasm32"))]
        inner.borrow_mut().start_render_thread(receiver, completed_sender);
        #[cfg(target_arch = "wasm32")]
        let _ = completed_sender;

        inner
    }

    pub fn request_redraw(&self) {
        self.redraw_requested.set(true);
        if let Some(winit_window) = &self.winit_window {
            winit_window.request_redraw();
        }
    }

    pub(crate) fn redraw_requested(&self) -> bool {
        self.redraw_requested.get() || self.renderer_redraw_requested.load(Ordering::Relaxed)
    }

    pub(crate) fn on_focused(&self, focused: bool) {
        {
            let root = self
                .element_data
                .access_root
                .expect("window accessibility root is not attached");
            if focused {
                let focus = crate::app::FOCUS.with(|focus| {
                    focus.borrow().as_ref().and_then(Weak::upgrade).and_then(|element| {
                        let element = element.borrow();
                        let data = element.element_data();
                        let belongs_to_window =
                            data.access_tree.ptr_eq(&self.access_tree) && data.access_root == Some(root);
                        belongs_to_window.then_some(data.access_key).flatten()
                    })
                });
                self.access_tree.set_focus(root, Some(focus.unwrap_or(root)));
            } else {
                self.access_tree.set_focus(root, None);
            }
        }
    }

    pub fn winit_window(&self) -> Option<Arc<dyn winit::window::Window>> {
        self.winit_window.clone()
    }

    pub fn set_winit_window(&mut self, window: Option<Arc<dyn WinitWindow>>) {
        self.winit_window = window;
    }

    /// Get the effective scale factor factoring window scale factor and zoom.
    pub fn effective_scale_factor(&self) -> f64 {
        self.scale_factor * self.zoom_scale_factor
    }

    /// Get the logical size of the window.
    pub fn window_size(&self) -> Size<f32> {
        Size::new(
            self.window_size.width / self.effective_scale_factor() as f32,
            self.window_size.height / self.effective_scale_factor() as f32,
        )
    }

    pub fn update_zoom(&mut self) {
        let scale_factor = self.effective_scale_factor();
        self.set_scale_factor(scale_factor);
    }

    pub(crate) fn zoom_in(&mut self) {
        self.zoom_scale_factor += 0.01;
        self.update_zoom();
    }

    pub(crate) fn zoom_out(&mut self) {
        let zoom_scale_factor = (self.zoom_scale_factor - 0.01).max(1.0);
        if self.zoom_scale_factor == zoom_scale_factor {
            return;
        }
        self.zoom_scale_factor = zoom_scale_factor;
        self.update_zoom();
    }

    pub(crate) fn maybe_zoom(&mut self, pointer_scroll_update: &PointerScrollEvent) -> bool {
        if self.modifiers.control_key() && pointer_scroll_update.pointer.pointer_type == PointerType::Mouse {
            let y: f32 = match pointer_scroll_update.delta {
                MouseScrollDelta::LineDelta(_, y) => y,
                MouseScrollDelta::PixelDelta(physical) => physical.y as f32,
                _ => 0.0,
            };
            if y < 0.0 {
                self.zoom_out();
            } else {
                self.zoom_in();
            }
            true
        } else {
            false
        }
    }

    pub(crate) fn maybe_zoom_keyboard(&mut self, keyboard_input: &KeyboardEvent) -> bool {
        if keyboard_input.modifiers.control_key() {
            if keyboard_input.key == "=" {
                self.zoom_in();
                return true;
            } else if keyboard_input.key == "-" {
                self.zoom_out();
                return true;
            }
        }
        false
    }

    pub(crate) fn tab_navigation_target(
        &self,
        keyboard_input: &KeyboardEvent,
    ) -> Option<Rc<RefCell<dyn ElementInternals>>> {
        if !keyboard_input.state.is_pressed()
            || keyboard_input.key != Key::Named(NamedKey::Tab)
            || keyboard_input.modifiers.control_key()
            || keyboard_input.modifiers.alt_key()
            || keyboard_input.modifiers.meta_key()
        {
            return None;
        }

        let mut navigation_elements = Vec::new();
        collect_tab_navigation_elements(&self.element_data.children, &mut navigation_elements);
        if navigation_elements.is_empty() {
            return None;
        }

        let current_focus = crate::app::FOCUS.with(|focus| focus.borrow().as_ref().and_then(Weak::upgrade));
        let current_index = current_focus.as_ref().and_then(|current| {
            navigation_elements
                .iter()
                .position(|(element, _)| Rc::ptr_eq(element, current))
        });
        let len = navigation_elements.len();
        let next_index = if let Some(current_index) = current_index {
            (1..=len)
                .map(|offset| {
                    if keyboard_input.modifiers.shift_key() {
                        (current_index + len - offset) % len
                    } else {
                        (current_index + offset) % len
                    }
                })
                .find(|index| navigation_elements[*index].1)
        } else if keyboard_input.modifiers.shift_key() {
            navigation_elements.iter().rposition(|(_, focusable)| *focusable)
        } else {
            navigation_elements.iter().position(|(_, focusable)| *focusable)
        };
        next_index.map(|index| navigation_elements[index].0.clone())
    }

    pub(crate) fn maybe_toggle_perf_stats(&mut self, keyboard_input: &KeyboardEvent) -> bool {
        if keyboard_input.repeat || !keyboard_input.state.is_pressed() {
            return false;
        }

        if keyboard_input.key != Key::Named(NamedKey::F3) {
            return false;
        }

        self.perf_stats.toggle();
        let enabled = !self.perf_stats.is_enabled();
        self.request_redraw();
        #[cfg(target_arch = "wasm32")]
        if let Some(renderer) = &mut self.renderer {
            renderer.set_vsync(enabled);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let sender = self.sender.clone();
            RetGuiRuntime::spawn_local(async move {
                sender.send(RenderMessage::SetVsync(enabled)).await.ok();
            });
        }
        true
    }

    pub(crate) fn perf_stats_enabled(&self) -> bool {
        self.perf_stats.is_enabled()
    }

    pub(crate) fn update_modifiers(&mut self, modifiers: ModifiersState) {
        self.modifiers = modifiers;
    }

    pub(crate) fn zoom_scale_factor(&self) -> f64 {
        self.zoom_scale_factor
    }

    pub(crate) fn mouse_position(&self) -> Option<Point> {
        self.mouse_positon
    }

    pub(crate) fn on_resize(&mut self, new_size: Size<f32>) {
        if self.window_size.width == new_size.width && self.window_size.height == new_size.height {
            return;
        }
        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            gummy_tree.mark_dirty(self.element_data.layout.gummy_node_id.unwrap());
        });

        self.window_size = new_size;
        self.resize_renderer_surface();
        self.request_redraw();
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn on_renderer_created(&mut self, mut renderer: Box<dyn Renderer + Send>, new_size: Size<f32>) {
        renderer.surface_set_clear_color(Color::WHITE);
        self.renderer = Some(renderer);
        if self.window_size.width != new_size.width || self.window_size.height != new_size.height {
            GUMMY_TREE.with_borrow_mut(|gummy_tree| {
                gummy_tree.mark_dirty(self.element_data.layout.gummy_node_id.unwrap());
            });
            self.window_size = new_size;
        }
        self.resize_renderer_surface();
        self.request_redraw();
    }

    fn resize_renderer_surface(&mut self) {
        #[cfg(target_arch = "wasm32")]
        if let Some(renderer) = &mut self.renderer {
            renderer.resize_surface(self.window_size.width.max(1.0), self.window_size.height.max(1.0));
        }
    }

    pub(crate) fn set_mouse_position(&mut self, point: Option<Point>) {
        self.mouse_positon = point;
    }

    pub(crate) fn on_redraw_requested(
        &mut self,
        text_context: &mut TextContext,
        resource_manager: Arc<ResourceManager>,
    ) {
        if let Some(frames) = &mut self.completed_frames {
            while let Ok(frame) = frames.try_recv() {
                self.perf_stats.update_stats(frame.total, frame.layout, frame.render);
            }
        }
        #[cfg(target_arch = "wasm32")]
        let buffer = {
            if self.renderer.is_none() {
                self.redraw_requested.set(true);
                return;
            }
            (0, std::mem::take(&mut self.render_list))
        };
        #[cfg(not(target_arch = "wasm32"))]
        let Some(buffer) = self.render_buffers.acquire() else {
            self.redraw_requested.set(true);
            return;
        };
        self.renderer_redraw_requested.store(false, Ordering::Relaxed);
        self.redraw_requested.set(false);
        let frame_start = Instant::now();
        let layout_stats = self.layout_window(text_context, resource_manager.clone());
        self.draw_window(text_context, resource_manager, frame_start, layout_stats, buffer);
    }

    pub(crate) fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        if self.scale_factor == scale_factor {
            return;
        }
        self.scale_factor = scale_factor;
        self.set_scale_factor(self.effective_scale_factor());
    }

    pub(crate) fn create(
        &mut self,
        retgui_app: &mut App,
        event_loop: Option<&dyn ActiveEventLoop>,
        _window: Weak<RefCell<Self>>,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.render_runtime = Some(retgui_app.runtime.handle());
        }
        let Some(event_loop) = event_loop else {
            if self.headless {
                return;
            }

            self.headless = true;
            let renderer =
                self.renderer_type
                    .create_headless(self.window_size.width, self.window_size.height);
            #[cfg(target_arch = "wasm32")]
            {
                self.renderer = Some(renderer);
                self.renderer.as_mut().unwrap().surface_set_clear_color(Color::WHITE);
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let sender = self.sender.clone();
                let resources = retgui_app.resource_manager.clone();
                retgui_app.runtime.maybe_block_on(async move {
                    sender.send(RenderMessage::SetRenderer(renderer, None)).await.expect("render worker stopped");
                    sender.send(RenderMessage::SetResourceManager(resources)).await.expect("render worker stopped");
                });
            }
            self.resize_renderer_surface();
            self.on_redraw_requested(
                retgui_app.text_context.borrow_mut().as_mut().unwrap(),
                retgui_app.resource_manager.clone(),
            );
            return;
        };

        self.headless = false;

        let winit_window: Arc<dyn WinitWindow> = Arc::from(if let Some(window_fn) = &mut self.advanced_window_fn {
            (*window_fn)(event_loop)
        } else {
            let window_attributes = WindowAttributes::default()
                .with_title(self.title.as_ref().unwrap())
                .with_visible(false);
            #[cfg(target_arch = "wasm32")]
            let window_attributes = {
                let canvas = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .get_element_by_id("canvas")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .unwrap();

                window_attributes
                    .with_platform_attributes(Box::new(WindowAttributesWeb::default().with_canvas(Some(canvas))))
            };

            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window")
        });
        self.set_winit_window(Some(winit_window.clone()));
        self.on_scale_factor_changed(winit_window.scale_factor());

        let renderer_type = self.renderer_type;
        cfg_select! {
            not(target_arch = "wasm32") => {
                let resource_manager = retgui_app.resource_manager.clone();
                retgui_app.runtime.borrow_tokio_runtime().block_on(async {
                    let mut renderer: Box<dyn Renderer + Send> = renderer_type.create(winit_window.clone()).await;
                    let pending = self.renderer_redraw_requested.clone();
                    let proxy = event_loop.create_proxy();
                    renderer.set_redraw_callback(Box::new(move || {
                        pending.store(true, Ordering::Relaxed);
                    }));
                    self.sender
                        .send(RenderMessage::SetRenderer(renderer, Some(proxy)))
                        .await
                        .expect("Failed to send renderer to renderer thread.");
                    self.sender
                        .send(RenderMessage::SetResourceManager(resource_manager))
                        .await
                        .expect("Failed to send resource manager to renderer thread.");
                });
            }
            _ => {
                let window_copy_2 = winit_window.clone();
                retgui_app.runtime.runtime_spawn(async move {
                    let renderer = renderer_type.create(window_copy_2.clone()).await;
                    if let Some(window) = _window.upgrade() {
                        let size = Size::new(
                            window_copy_2.surface_size().width as f32,
                            window_copy_2.surface_size().height as f32,
                        );
                        window.borrow_mut().on_renderer_created(renderer, size);
                    }
                    info!("Created renderer")
                });
            }
        }

        {
            self.on_redraw_requested(
                retgui_app.text_context.borrow_mut().as_mut().unwrap(),
                retgui_app.resource_manager.clone(),
            );
            let root = self
                .element_data
                .access_root
                .expect("window accessibility root is not attached");
            self.access_tree.set_root_window(root, winit_window.clone());
            self.access_tree.set_focus(root, Some(root));
            self.access_tree.register_window(winit_window.clone());
        }

        winit_window.set_visible(true);
    }

    fn layout_window(&mut self, text_context: &mut TextContext, resource_manager: Arc<ResourceManager>) -> LayoutStats {
        let total_start = Instant::now();
        let mut compute = Duration::from_secs(0);
        let mut apply = Duration::from_secs(0);

        let root_node = self
            .element_data
            .layout
            .gummy_node_id
            .expect("A root element must have a layout node.");

        let window_size = self.window_size();
        let available_space: gummy::Size<AvailableSpace> = gummy::Size {
            width: AvailableSpace::Definite(window_size.width),
            height: AvailableSpace::Definite(window_size.height),
        };

        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            let root_dirty = gummy_tree.is_layout_dirty(root_node);

            if root_dirty {
                let compute_start = Instant::now();
                gummy_tree.compute_layout(root_node, available_space, text_context, resource_manager.clone());
                compute = compute_start.elapsed();
            }

            if root_dirty || gummy_tree.is_apply_layout_dirty(&root_node) {
                let apply_start = Instant::now();
                let sf = self.effective_scale_factor();
                let owners = gummy_tree.take_layout_owners(root_node, root_dirty);
                for (owner_id, owner, layout_order) in owners {
                    let mut layout_order = layout_order;
                    if owner_id == self.element_data.internal_id {
                        self.apply_layout(gummy_tree, &mut layout_order, text_context, sf);
                    } else {
                        owner
                            .borrow_mut()
                            .apply_layout(gummy_tree, &mut layout_order, text_context, sf);
                    }
                }
                gummy_tree.apply_layout(root_node);
                apply = apply_start.elapsed();
            }
            //}
        });

        LayoutStats::new(total_start.elapsed(), compute, apply)
    }

    fn draw_window(
        &mut self,
        text_context: &mut TextContext,
        resource_manager: Arc<ResourceManager>,
        frame_start: Instant,
        layout_stats: LayoutStats,
        (_buffer_index, mut render_list): (usize, RenderList),
    ) {
        let total_start = Instant::now();

        render_list.set_cull(Some(Rectangle::new(
            0.0,
            0.0,
            self.window_size.width,
            self.window_size.height,
        )));
        let scale_factor = self.effective_scale_factor();

        self.draw_transformed(&mut render_list, resource_manager.clone(), scale_factor, text_context);
        let build_list = total_start.elapsed();
        let debug_overlay_start = Instant::now();
        self.perf_stats.draw(&mut render_list, text_context, scale_factor);
        let debug_overlay = debug_overlay_start.elapsed();

        if let Some(winit_window) = &self.winit_window {
            winit_window.pre_present_notify();
        }

        {
            let mut targets = self.hit_test_targets.borrow_mut();
            std::mem::swap(&mut *targets, &mut render_list.targets);
            render_list.targets.clear();

            TargetItem::sort_items_by_overlay_depth(&mut targets);
        }
        let frame = FrameStats {
            start: frame_start,
            total: Duration::ZERO,
            layout: layout_stats,
            render: RenderStats::new(
                total_start.elapsed(),
                build_list,
                debug_overlay,
                Duration::ZERO,
                Duration::ZERO,
                Duration::ZERO,
            ),
        };
        #[cfg(target_arch = "wasm32")]
        {
            let mut frame = frame;
            let renderer = self.renderer.as_mut().expect("renderer is not initialized");
            let window = Rectangle::new(0.0, 0.0, renderer.surface_width(), renderer.surface_height());
            let start = Instant::now();
            render_list.sort_render_list();
            let sort = start.elapsed();
            let start = Instant::now();
            renderer.prepare(&mut render_list, resource_manager, window);
            let prepare = start.elapsed();
            let start = Instant::now();
            renderer.submit();
            frame.finish_render(sort, prepare, start.elapsed());
            self.perf_stats.update_stats(frame.total, frame.layout, frame.render);
            render_list.clear();
            self.render_list = render_list;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let sender = self.sender.clone();
            let buffers = Arc::clone(&self.render_buffers);
            let size = self.window_size;
            RetGuiRuntime::spawn_local(async move {
                buffers.queue(_buffer_index, render_list).await;
                if sender.send(RenderMessage::DoRender(_buffer_index, frame, size)).await.is_err() {
                    let list = buffers.take_queued(_buffer_index).await;
                    buffers.recycle(_buffer_index, list).await;
                }
            });
        }

        if self.perf_stats.is_enabled() {
            self.request_redraw();
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn screenshot(&mut self) -> Screenshot {
        self.renderer.as_ref().expect("window must be created before taking a screenshot").screenshot()
    }

    pub(crate) fn close_internal(&mut self, _runtime: &RetGuiRuntime) {
        cfg_select! {
            not(target_arch = "wasm32") => {
                let sender = self.sender.clone();
                _runtime.maybe_block_on(async move {
                    info!("Sent");
                    sender.send(RenderMessage::Close).await.ok();
                })
            }
            _ => self.renderer = None,
        }
        self.winit_window = None;
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_render_thread(
        &self,
        mut receiver: Receiver<RenderMessage>,
        completed_sender: UnboundedSender<FrameStats>,
    ) {
        let buffers = Arc::clone(&self.render_buffers);
        thread::spawn(|| {
            let rt = LocalRuntime::new().unwrap();
            rt.block_on(async move {
                let mut renderer: Option<Box<dyn Renderer>> = None;
                let mut redraw_proxy = None;
                let mut resource_manager: Option<Arc<ResourceManager>> = None;
                while let Some(message) = receiver.recv().await {
                    match message {
                        RenderMessage::SetRenderer(mut new_renderer, proxy) => {
                            new_renderer.surface_set_clear_color(Color::WHITE);
                            renderer = Some(new_renderer);
                            redraw_proxy = proxy;
                        }
                        RenderMessage::SetResourceManager(new_resource_manager) => {
                            resource_manager = Some(new_resource_manager);
                        }
                        RenderMessage::SetVsync(enabled) => {
                            if let Some(renderer) = &mut renderer {
                                renderer.set_vsync(enabled);
                            }
                        }
                        RenderMessage::Screenshot(reply) => {
                            reply.send(renderer.as_ref().expect("renderer is not initialized").screenshot()).ok();
                        }
                        RenderMessage::DoRender(buffer_index, mut frame, size) => {
                            let mut render_list = buffers.take_queued(buffer_index).await;
                            if let Some(renderer) = &mut renderer
                                && let Some(resource_manager) = &mut resource_manager
                            {
                                let width = size.width.max(1.0);
                                let height = size.height.max(1.0);
                                if renderer.surface_width() != width || renderer.surface_height() != height {
                                    renderer.resize_surface(width, height);
                                }
                                let window =
                                    Rectangle::new(0.0, 0.0, renderer.surface_width(), renderer.surface_height());
                                let start = Instant::now();
                                render_list.sort_render_list();
                                let sort = start.elapsed();
                                let start = Instant::now();
                                renderer.prepare(&mut render_list, resource_manager.clone(), window);
                                let prepare = start.elapsed();
                                let start = Instant::now();
                                renderer.submit();
                                let submit = start.elapsed();
                                buffers.recycle(buffer_index, render_list).await;
                                frame.finish_render(sort, prepare, submit);
                                completed_sender.send(frame).ok();
                            } else {
                                buffers.recycle(buffer_index, render_list).await;
                            }
                            if let Some(proxy) = &redraw_proxy {
                                proxy.wake_up();
                            }
                        }
                        RenderMessage::Close => {
                            info!("Closing");
                            break;
                        }
                    }
                }
            });
        });
    }
}

fn collect_tab_navigation_elements(
    children: &[DynElement],
    navigation_elements: &mut Vec<(Rc<RefCell<dyn ElementInternals>>, bool)>,
) {
    for child in children {
        let (is_visible, is_keyboard_focusable, grandchildren) = {
            let child = child.borrow();
            (
                child.is_visible(),
                child.is_keyboard_focusable(),
                child.element_data().children.clone(),
            )
        };

        if !is_visible {
            continue;
        }
        navigation_elements.push((child.inner.clone(), is_keyboard_focusable));
        collect_tab_navigation_elements(&grandchildren, navigation_elements);
    }
}
