use std::collections::VecDeque;
use std::sync::Arc;

use retgui::elements::{DynElement, Element, ElementData, ElementIds, ElementInternals, HasElementData, RetGuiAccessTree, RetainedElements, Text, Window, clone_element};
use retgui::events::EventKind;
use retgui::layout::GummyTree;
use retgui::style::AlignSelf;
use retgui::text::text_context::TextContext;
use retgui::{App, Brush, Color, Renderer, ResourceManager, RetGuiOptions, pct, px, retgui_main, rgb};

use util::setup_logging;

#[derive(Clone, Copy)]
struct ColorTile {
    inner: DynElement,
}

#[derive(Clone)]
struct ColorTileElement {
    element_data: ElementData,
    color: Color,
    alternate: Color,
    clicks: u32,
}

impl Element for ColorTile {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl HasElementData for ColorTileElement {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for ColorTileElement {
    fn deep_clone(
        &self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
    ) -> DynElement {
        clone_element(self, elements, gummy_tree, access_tree, by_internal_id, |_, _| None)
    }

    fn draw(
        &self,
        elements: &RetainedElements,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        if !self.is_visible() {
            return;
        }

        self.add_hit_testable(renderer, true, scale_factor);
        self.draw_borders(renderer, scale_factor);
        let bounds = self.computed_box().content_rectangle().scale(scale_factor);
        renderer.draw_rect(bounds, Brush::Color(self.color));
        self.draw_children(elements, renderer, resource_manager, scale_factor, text_context);
    }

    fn on_event(
        &mut self,
        _elements: &mut RetainedElements,
        _gummy_tree: &mut GummyTree,
        _access_tree: &RetGuiAccessTree,
        _by_internal_id: &mut ElementIds,
        _event_queue: &mut VecDeque<EventKind>,
        _focus: &mut Option<DynElement>,
        _focus_outline_visible: bool,
        _pending_animation_updates: &mut Vec<(DynElement, bool)>,
        event: &mut EventKind,
        _text_context: &mut TextContext,
    ) {
        if matches!(event, EventKind::Click(_)) {
            std::mem::swap(&mut self.color, &mut self.alternate);
            self.clicks += 1;
            self.request_window_redraw();
        }
    }
}

impl ColorTile {
    fn new<S: 'static>(app: &mut App<S>) -> Self {
        let inner = app.insert_element(true, |element_data| ColorTileElement {
            element_data,
            color: rgb(37, 99, 235),
            alternate: rgb(219, 39, 119),
            clicks: 0,
        });
        Self { inner }
    }

    fn fill_color<S: 'static>(self, app: &mut App<S>, color: Color) -> Self {
        let tile = app.get_as_mut::<ColorTileElement>(self.inner);
        tile.color = color;
        tile.request_window_redraw();
        self
    }

    fn click_count<S: 'static>(self, app: &App<S>) -> u32 {
        app.get_as::<ColorTileElement>(self.inner).clicks
    }
}

fn main() {
    setup_logging();

    let mut app = App::new();
    let label = Text::new(&mut app, "Click the custom Element");
    label.set_font_size(&mut app, 20.0);
    label.set_selectable(&mut app, false);

    let tile = ColorTile::new(&mut app);
    tile.fill_color(&mut app, rgb(37, 99, 235));
    tile.set_align_self(&mut app, AlignSelf::Start);
    tile.set_width(&mut app, px(320));
    tile.set_height(&mut app, px(180));
    tile.set_padding_all(&mut app, px(24));
    tile.set_border_radius_all(&mut app, (16.0, 16.0));
    tile.push(&mut app, label);

    assert_eq!(tile.click_count(&app), 0);

    let window = Window::new(&mut app, "Custom Element");
    window.set_width(&mut app, pct(100));
    window.set_height(&mut app, pct(100));
    window.push(&mut app, tile);

    retgui_main(app, (), RetGuiOptions::basic("Custom Element"));
}
