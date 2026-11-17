use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time;

use gummy::{AvailableSpace, Size};

use parley::{AlignmentOptions, ContentWidths, Selection};

use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::{Point, Rectangle, Vec2};

use retgui_renderer::RenderList;

use retgui_resource_manager::ResourceManager;

use rustc_hash::FxHashMap;

use smol_str::{SmolStr, ToSmolStr};

use time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
use web_time as time;

use crate::elements::element_data::ElementData;
use crate::elements::traits::clone_element;
use crate::elements::{AsElement, DynElement, Element, ElementInternals};
use crate::events::{Event, EventKind, KeyboardEvent, PointerButton, PointerButtonEvent, PointerMovedEvent};
use crate::layout::GummyTree;
use crate::layout::layout_context::{GummyTextContext, LayoutContext, TextHashKey};
use crate::style::{Style, TextAlign};
use crate::text::text_context::TextContext;
use crate::text::text_render_data;
use crate::text::text_render_data::TextRender;
use winit::dpi;

const MAX_CACHE_SIZE: usize = 16;

/// Displays plain selectable text.
#[derive(Clone)]
pub struct Text {
    pub(crate) inner: Rc<RefCell<TextInner>>,
}

#[derive(Clone)]
pub(crate) struct TextInner {
    element_data: ElementData,
    /// Whether the text can be selected.
    selectable: bool,
    /// Underlying text data.
    pub(crate) state: TextState,
    /// A weak reference to this element used for the gummy layout tree.
    me: Weak<RefCell<Self>>,
}

/// Text layout, render data, and state for the element's text.
#[derive(Clone)]
pub struct TextState {
    /// The laid out glyphs, lines, and cursors needed to draw the text.
    /// This includes all text data and is not culled.
    pub(crate) text_render: Option<Arc<TextRender>>,
    /// The last time the text was clicked.
    pub(crate) last_click_time: Option<Instant>,
    /// Click count used to determine how text should be selected e.g. words, lines, etc.
    pub(crate) click_count: u32,
    /// True if the pointer is pressed on the element.
    pub(crate) pointer_down: bool,
    /// The last time the cursor animation started.
    pub(crate) start_time: Option<Instant>,
    /// Blink period used for animating the text cursor.
    pub(crate) blink_period: Duration,
    /// The underlying string for this text element.
    text: SmolStr,
    /// The scale factor including zoom and display scaling.
    scale_factor: f64,
    /// Alignment of the text.
    text_align: TextAlign,
    /// The selected text range. Multiple selections are not yet supported.
    selection: Selection,
    layout: Option<parley::Layout<Brush>>,
    /// Cache used to store the computed size of text.
    measure_cache: FxHashMap<TextHashKey, Size<f32>>,
    /// The last measure key. If we have a new key, we need to re-layout the text.
    current_layout_key: Option<TextHashKey>,
    /// The last measure key requested during layout.
    /// Multiple measures may happen during layout, but only the final one needs rendered.
    last_requested_measure_key: Option<TextHashKey>,
    /// The last render key. If we have a new key, we need to re-render the layout out text.
    current_render_key: Option<TextHashKey>,
    content_widths: Option<ContentWidths>,
    /// True if the rendered text should use glyph caching.
    /// Can cause performance issues for animated text.
    use_glyph_cache: bool,
    /// The last known cursor position.
    ///
    /// The cursor is assumed to start at (0.0, 0.0). The cursor_pos may return points
    /// outside the text input.
    cursor_pos: Point,
    /// True if the layout needs to be recomputed for any reason.
    is_layout_dirty: bool,
}

impl Element for Text {}

impl Drop for TextInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for Text {
    fn with<R>(&self, callback: impl FnOnce(&dyn ElementInternals) -> R) -> R {
        callback(&*self.inner.borrow())
    }

    fn with_mut<R>(&self, callback: impl FnOnce(&mut dyn ElementInternals) -> R) -> R {
        callback(&mut *self.inner.borrow_mut())
    }
}

impl crate::elements::ElementData for TextInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            text: SmolStr::new(""),
            scale_factor: 1.0,
            text_align: Default::default(),
            selection: Selection::default(),
            text_render: None,
            layout: None,
            measure_cache: Default::default(),
            current_layout_key: None,
            last_requested_measure_key: None,
            current_render_key: None,
            content_widths: None,
            use_glyph_cache: true,
            last_click_time: None,
            click_count: 0,
            pointer_down: false,
            cursor_pos: Point::new(0.0, 0.0),
            start_time: None,
            blink_period: Default::default(),
            is_layout_dirty: false,
        }
    }
}

impl ElementInternals for TextInner {
    fn deep_clone(&self) -> DynElement {
        DynElement::new(clone_element::<Self, _>(self, |element, gummy_tree| {
            let me = Rc::downgrade(element);
            let mut element = element.borrow_mut();
            element.me = me;
            let node = element.element_data.layout.gummy_node_id();
            let context = LayoutContext::Text(GummyTextContext {
                element: element.me.clone(),
            });
            gummy_tree.set_node_context(node, Some(context));
            Some(node)
        }))
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        let node = self.element_data.layout.gummy_node_id.unwrap();
        let result = gummy_tree.get_layout(node);
        let has_new_layout = gummy_tree.has_new_layout(node);

        self.element_data.layout.has_new_layout = has_new_layout;
        if has_new_layout {
            self.resolve_box(result, z_index);
            self.apply_borders(scale_factor);
        }

        if has_new_layout {
            gummy_tree.mark_seen(node);
        }

        // It is slower to cache animating glyphs.
        let use_glyph_cache = !self
            .element_data
            .animations
            .iter()
            .any(|animation| !animation.is_finished() && animation.animates_font_size());

        let state: &mut TextState = &mut self.state;
        if state.current_layout_key != state.last_requested_measure_key {
            state.layout(
                state.last_requested_measure_key.unwrap().known_dimensions(),
                state.last_requested_measure_key.unwrap().available_space(),
            );
        }

        state.try_update_text_render(
            text_context,
            self.element_data.style.get_selection_brush(),
            self.element_data.style.get_text_brush(),
            use_glyph_cache,
        );
    }

    fn draw(
        &mut self,
        render_list: &mut RenderList,
        _resource_manager: Arc<ResourceManager>,
        _scale_factor: f64,
        _text_context: &mut TextContext,
    ) {
        if !self.is_visible() {
            return;
        }
        self.maybe_start_overlay(render_list);
        self.add_hit_testable(render_list, true, _scale_factor);
        let content_rectangle = self.element_data.layout.local_box().content_rectangle();
        self.draw_borders(render_list, _scale_factor);
        render_list.draw_text(
            self.state.text_render.clone().unwrap(),
            content_rectangle.scale(_scale_factor),
            None,
            false,
            self.state.use_glyph_cache,
        );
        self.maybe_end_overlay(render_list);
    }

    fn on_event(&mut self, event: &mut EventKind, _text_context: &mut TextContext) {
        if !self.selectable {
            return;
        }
        match event {
            EventKind::PointerDown(event) if event.button == Some(PointerButton::Left) => {
                self.handle_pointer_down(event);
            }
            EventKind::PointerUp(event) if event.button == Some(PointerButton::Left) => {
                self.handle_pointer_up(event);
            }
            EventKind::PointerMoved(event) => {
                self.handle_pointer_move(event);
            }
            EventKind::KeyDown(event) => {
                self.handle_key_down(event);
            }
            _ => {}
        }
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.element_data.applied_scale_factor = scale_factor;
        self.apply_borders(scale_factor);
        self.state.is_layout_dirty = true;
        self.mark_dirty();
        self.state.scale_factor = scale_factor;
    }

    fn set_text_brush(&mut self, brush: Brush) {
        self.style_mut().set_text_brush(brush.clone());
        if let Some(text_render) = &mut self.state.text_render {
            let text_render = Arc::make_mut(text_render);
            text_render.override_brush = Some(brush);
        }
        self.request_window_redraw();
    }

    fn on_text_style_changed(&mut self) {
        self.state.is_layout_dirty = true;
    }

    fn set_selection_brush(&mut self, selection_brush: Brush) {
        self.style_mut().set_selection_brush(selection_brush.clone());
        self.state.update_text_selection(selection_brush);
        self.request_window_redraw();
    }
}

impl Text {
    /// Create a selectable text element with a given string.
    pub fn new(text: &str) -> Self {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<TextInner>>| {
            RefCell::new(TextInner {
                element_data: ElementData::new(me.clone(), false),
                selectable: true,
                state: TextState::default(),
                me: me.clone(),
            })
        });
        let mut inner_mut = inner.borrow_mut();

        inner_mut.element_data.set_accessibility_role(issho::Role::Label);
        let selectable = inner_mut.selectable;
        inner_mut.set_selectable(selectable);

        let text_context = Some(LayoutContext::Text(GummyTextContext {
            element: inner_mut.me.clone(),
        }));
        inner_mut.element_data.create_layout_node(text_context);
        inner_mut.set_text(text);

        drop(inner_mut);
        Text { inner }
    }

    /// Set whether the text is selectable.
    pub fn set_selectable(self, selectable: bool) -> Self {
        self.inner.borrow_mut().set_selectable(selectable);
        self
    }

    /// Return whether the text is selectable.
    pub fn selectable(&self) -> bool {
        self.inner.borrow().selectable()
    }

    /// Get the underlying text content regardless of selection.
    pub fn text(&self) -> String {
        self.inner.borrow().text().to_owned()
    }

    /// Set the content of the text. Layout and render will happen during the next frame.
    pub fn set_text(self, text: &str) -> Self {
        self.inner.borrow_mut().set_text(text);
        self
    }

    /// Set the content of the text. Layout and render will happen during the next frame.
    pub fn set_text_smol_str(self, text: SmolStr) -> Self {
        self.inner.borrow_mut().set_text_smol_str(text);
        self
    }
}

impl TextInner {
    /// Return whether the text is selectable.
    pub fn selectable(&self) -> bool {
        self.selectable
    }

    /// Set whether the text is selectable.
    pub fn set_selectable(&mut self, selectable: bool) -> &mut Self {
        self.selectable = selectable;
        self.element_data.set_selectable(self.selectable);
        self
    }

    /// Get the underlying text content regardless of selection.
    pub fn text(&self) -> &str {
        &self.state.text
    }

    /// Set the text.
    ///
    /// Updates the text content immediately. Mark layout and render caches as dirty. Layout and
    /// render caches will be computed in the next layout/render pass.
    pub fn set_text(&mut self, text: &str) -> &mut Self {
        self.set_text_smol_str(text.to_smolstr());
        self
    }

    /// Set the text.
    ///
    /// Updates the text content immediately. Mark layout and render caches as dirty. Layout and
    /// render caches will be computed in the next layout/render pass.
    pub fn set_text_smol_str(&mut self, text: SmolStr) {
        self.state.text = text;
        self.state.is_layout_dirty = true;
        self.mark_dirty();
        self.element_data.set_accessibility_name(self.state.text.clone());
    }

    /// Measure the size of the text given the current styles and layout constraints.
    pub(crate) fn measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        text_context: &mut TextContext,
    ) -> Size<f32> {
        self.state.measure(
            &self.element_data.style,
            known_dimensions,
            available_space,
            text_context,
        )
    }

    /// Handles pointer down. This involves text selection.
    fn handle_pointer_down(&mut self, event: &mut PointerButtonEvent) {
        self.focus();

        let state: &mut TextState = &mut self.state;
        let previous_selection = (state.selection.anchor(), state.selection.focus());
        state.pointer_down = true;
        state.cursor_reset();
        let now = Instant::now();
        if let Some(last) = state.last_click_time.take() {
            if now.duration_since(last).as_secs_f64() < 0.25 {
                state.click_count = (state.click_count + 1) % 4;
            } else {
                state.click_count = 1;
            }
        } else {
            state.click_count = 1;
        }
        state.last_click_time = Some(now);
        let click_count = state.click_count;
        let cursor_pos = state.cursor_pos;
        match click_count {
            2 => state.select_word_at_point(cursor_pos),
            3 => state.select_line_at_point(cursor_pos),
            _ => state.move_to_point(cursor_pos),
        }
        let selection_changed = previous_selection != (state.selection.anchor(), state.selection.focus());
        if selection_changed {
            state.update_text_selection(self.element_data.style.get_selection_brush());
        }
        if click_count == 1 {
            self.set_pointer_capture(event.pointer.pointer_id.unwrap());
        }
        if selection_changed {
            self.request_window_redraw();
        }
        event.prevent_default();
    }

    /// Handles pointer up. This involves text selection.
    fn handle_pointer_up(&mut self, event: &mut PointerButtonEvent) {
        self.state.pointer_down = false;
        self.state.cursor_reset();
        self.release_pointer_capture(event.pointer.pointer_id.unwrap());
        event.prevent_default();
    }

    /// Handles pointer move. This involves text selection.
    fn handle_pointer_move(&mut self, event: &mut PointerMovedEvent) {
        let text_position = self.get_computed_box_transformed().content_rectangle();
        let prev_pos = self.state.cursor_pos;
        // NOTE: Cursor position should be relative to the top left of the text box.
        self.state.cursor_pos =
            event.current.logical_point() - Vec2::new(text_position.x as f64, text_position.y as f64);
        // macOS seems to generate a spurious move after selecting word?
        if self.state.pointer_down && prev_pos != self.state.cursor_pos {
            let previous_selection = (self.state.selection.anchor(), self.state.selection.focus());
            self.state.cursor_reset();
            let cursor_pos = self.state.cursor_pos;
            self.state.extend_selection_to_point(cursor_pos);
            if previous_selection != (self.state.selection.anchor(), self.state.selection.focus()) {
                self.state
                    .update_text_selection(self.element_data.style.get_selection_brush());
                self.request_window_redraw();
            }
        }
        event.prevent_default();
    }

    #[cfg(all(
        any(target_os = "windows", target_os = "macos", target_os = "linux"),
        feature = "clipboard"
    ))]
    fn handle_ctrl_c(&mut self, event: &mut KeyboardEvent) {
        let copy_modifier = if cfg!(target_os = "macos") {
            event.modifiers.meta_key()
        } else {
            event.modifiers.control_key()
        };
        let is_c_key = matches!(&event.key, winit::keyboard::Key::Character(c) if c.eq_ignore_ascii_case("c"));
        let range = self.state.selection.text_range();
        let is_copy_key_combo = self.is_focused() && !event.is_composing && copy_modifier && is_c_key;
        if is_copy_key_combo && !range.is_empty() {
            let selected_text = self.state.text.get(range);
            if let Some(text) = selected_text
                && crate::clipboard::copy(text.to_owned())
            {
                event.prevent_default();
                event.stop_propagation();
            }
        }
    }

    /// Handles keyboard selection.
    fn handle_keyboard_selection(&mut self, event: &mut KeyboardEvent) {
        if !self.is_focused() || event.is_composing {
            return;
        }
        let Some(layout) = self.state.layout.as_ref() else {
            return;
        };

        let extend = event.modifiers.shift_key();
        let is_mac = cfg!(target_os = "macos");
        let action_modifier = if is_mac {
            event.modifiers.meta_key()
        } else {
            event.modifiers.control_key()
        };
        let word_modifier = if is_mac {
            event.modifiers.alt_key()
        } else {
            event.modifiers.control_key()
        };
        let previous_selection = (self.state.selection.anchor(), self.state.selection.focus());
        let selection = &self.state.selection;
        self.state.selection = match &event.key {
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft) => {
                if is_mac && action_modifier {
                    selection.line_start(layout, extend)
                } else if word_modifier {
                    selection.previous_visual_word(layout, extend)
                } else {
                    selection.previous_visual(layout, extend)
                }
            }
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight) => {
                if is_mac && action_modifier {
                    selection.line_end(layout, extend)
                } else if word_modifier {
                    selection.next_visual_word(layout, extend)
                } else {
                    selection.next_visual(layout, extend)
                }
            }
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                if is_mac && action_modifier {
                    selection.move_lines(layout, isize::MIN, extend)
                } else if word_modifier {
                    selection
                        .previous_visual(layout, extend)
                        .hard_line_start(layout, extend)
                } else {
                    selection.previous_line(layout, extend)
                }
            }
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
                if is_mac && action_modifier {
                    selection.move_lines(layout, isize::MAX, extend)
                } else if word_modifier {
                    selection.hard_line_end(layout, extend).next_visual(layout, extend)
                } else {
                    selection.next_line(layout, extend)
                }
            }
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Home) => {
                if action_modifier {
                    selection.move_lines(layout, isize::MIN, extend)
                } else {
                    selection.line_start(layout, extend)
                }
            }
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::End) => {
                if action_modifier {
                    selection.move_lines(layout, isize::MAX, extend)
                } else {
                    selection.line_end(layout, extend)
                }
            }
            _ => return,
        };

        self.state.cursor_reset();
        if previous_selection != (self.state.selection.anchor(), self.state.selection.focus()) {
            self.state
                .update_text_selection(self.element_data.style.get_selection_brush());
            self.request_window_redraw();
        }
        event.prevent_default();
        event.stop_propagation();
    }

    /// Handles key down, which is typically ctrl + c and keyboard based text selection.
    fn handle_key_down(&mut self, event: &mut KeyboardEvent) {
        #[cfg(all(
            any(target_os = "windows", target_os = "macos", target_os = "linux"),
            feature = "clipboard"
        ))]
        self.handle_ctrl_c(event);
        if event.is_propagation_stopped() {
            return;
        }
        self.handle_keyboard_selection(event);
    }
}

impl TextState {
    /// Measure the size of the text given the current styles and layout constraints.
    pub fn measure(
        &mut self,
        style: &Style,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        text_context: &mut TextContext,
    ) -> Size<f32> {
        if self.is_layout_dirty {
            self.clear_cache();
        }

        if self.layout.is_none() {
            let mut builder = text_context.tree_builder(self.scale_factor as f32, &style.to_text_style());
            let text = &self.text;
            builder.push_text(text);
            let (layout, _) = builder.build();
            self.content_widths = Some(layout.calculate_content_widths());
            self.layout = Some(layout);
        }

        self.text_align = style.get_text_align();

        let key = TextHashKey::new(known_dimensions, available_space);

        self.last_requested_measure_key = Some(key);
        if let Some(value) = self.measure_cache.get(&key) {
            return *value;
        }

        self.layout(known_dimensions, available_space)
    }

    /// Calculates the constraints from known dimensions and available space.
    fn compute_constraints(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<Option<f32>> {
        let known_dimensions = known_dimensions.map(|dimension| {
            dimension.map(|value| dpi::PhysicalUnit::from_logical::<f32, f32>(value, self.scale_factor).0)
        });
        let content_widths = self.content_widths.as_ref().unwrap();
        let width_constraint = known_dimensions.width.or(match available_space.width {
            AvailableSpace::MinContent => Some(content_widths.min),
            AvailableSpace::MaxContent => Some(content_widths.max),
            AvailableSpace::Definite(width) => {
                let scaled_width: f32 = dpi::PhysicalUnit::from_logical::<f32, f32>(width, self.scale_factor).0;
                Some(scaled_width.max(content_widths.min))
            }
        });
        let height_constraint = known_dimensions.height.or(match available_space.height {
            AvailableSpace::MinContent => None,
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(height) => {
                let scaled_height = dpi::PhysicalUnit::from_logical::<f32, f32>(height, self.scale_factor).0;
                Some(scaled_height)
            }
        });
        Size {
            width: width_constraint,
            height: height_constraint,
        }
    }

    /// Lays out the text with the given size constraints.
    fn layout_text(&mut self, constraints: Size<Option<f32>>) -> Size<f32> {
        let alignment = self.text_align.to_parley_alignment();
        let layout = self.layout.as_mut().unwrap();
        layout.set_alignment(
            alignment,
            AlignmentOptions {
                align_when_overflowing: true,
                ..Default::default()
            },
        );
        layout.break_all_lines(constraints.width);
        self.selection = self.selection.refresh(layout);
        let height = layout.height().min(constraints.height.unwrap_or(f32::MAX));
        let size = dpi::PhysicalSize::new(layout.width(), height).to_logical::<f32>(self.scale_factor);
        Size {
            width: size.width,
            height: size.height,
        }
    }

    /// Caches the laid out text dimensions for a given key.
    fn cache_layout(&mut self, key: TextHashKey, size: Size<f32>) {
        if self.measure_cache.len() >= MAX_CACHE_SIZE && !self.measure_cache.contains_key(&key) {
            // TODO: Use LRU?
            let evicted_key = *self.measure_cache.keys().next().unwrap();
            self.measure_cache.remove(&evicted_key);
        }
        self.measure_cache.insert(key, size);
        self.current_layout_key = Some(key);
    }

    /// Lays out the text under the given constraints.
    pub fn layout(&mut self, known_dimensions: Size<Option<f32>>, available_space: Size<AvailableSpace>) -> Size<f32> {
        let constraints = self.compute_constraints(known_dimensions, available_space);
        let size = self.layout_text(constraints);
        let key = TextHashKey::new(known_dimensions, available_space);
        self.cache_layout(key, size);
        size
    }

    /// Use the text layout to update the rendered text data.
    pub fn try_update_text_render(
        &mut self,
        _text_context: &mut TextContext,
        selection_brush: Brush,
        text_brush: Brush,
        use_glyph_cache: bool,
    ) {
        self.use_glyph_cache = use_glyph_cache;

        if self.current_render_key == self.current_layout_key {
            return;
        }

        let layout = self.layout.as_ref().unwrap();
        let mut text_render = text_render_data::from_editor(layout);
        text_render.override_brush = Some(text_brush);
        self.text_render = Some(Arc::new(text_render));
        self.current_render_key = self.current_layout_key;

        self.update_text_selection(selection_brush);
    }

    /// Reset the cursor blink animation timings.
    pub fn cursor_reset(&mut self) {
        self.start_time = Some(Instant::now());
        self.blink_period = Duration::from_millis(500);
    }

    /// Extend the text selection to the point.
    pub fn extend_selection_to_point(&mut self, point: Point) {
        let scale_factor = self.layout.as_ref().unwrap().scale() as f64;
        let point = Point::new(point.x * scale_factor, point.y * scale_factor);
        self.selection = self
            .selection
            .extend_to_point(self.layout.as_ref().unwrap(), point.x as f32, point.y as f32);
    }

    /// Selects a word at the given point.
    pub fn select_word_at_point(&mut self, point: Point) {
        let scale_factor = self.layout.as_ref().unwrap().scale() as f64;
        let point = Point::new(point.x * scale_factor, point.y * scale_factor);
        self.selection = Selection::word_from_point(self.layout.as_ref().unwrap(), point.x as f32, point.y as f32);
    }

    /// Selects the line the given point.
    pub fn select_line_at_point(&mut self, point: Point) {
        let scale_factor = self.layout.as_ref().unwrap().scale() as f64;
        let point = Point::new(point.x * scale_factor, point.y * scale_factor);
        self.selection = Selection::line_from_point(self.layout.as_ref().unwrap(), point.x as f32, point.y as f32);
    }

    /// Moves the cursor to a single point.
    pub fn move_to_point(&mut self, point: Point) {
        let scale_factor = self.layout.as_ref().unwrap().scale() as f64;
        let point = Point::new(point.x * scale_factor, point.y * scale_factor);
        self.selection = Selection::from_point(self.layout.as_ref().unwrap(), point.x as f32, point.y as f32);
    }

    /// Clear all the text caches and rendered text data.
    pub fn clear_cache(&mut self) {
        self.layout = None;
        self.measure_cache.clear();
        self.current_layout_key = None;
        self.last_requested_measure_key = None;
        self.current_render_key = None;
        self.text_render = None;
        self.content_widths = None;
        self.is_layout_dirty = false;
    }

    /// Update the rendered text data with the highlighted text region.
    ///
    /// This is typically called after the text selection changes.
    fn update_text_selection(&mut self, selection_brush: Brush) {
        if let Some(layout) = self.layout.as_ref() {
            let text_renderer = self.text_render.as_mut().unwrap();
            let text_renderer = Arc::make_mut(text_renderer);
            for line in text_renderer.lines.iter_mut() {
                line.selections.clear();
            }
            self.selection.geometry_with(layout, |rect, line| {
                text_renderer.lines[line].selections.push((
                    Rectangle::new(
                        rect.x0 as f32,
                        rect.y0 as f32,
                        rect.width() as f32,
                        rect.height() as f32,
                    ),
                    selection_brush.clone(),
                ));
            });
        }
    }
}
