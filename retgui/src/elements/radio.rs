//! A selectable circle.

use std::any::Any;
use std::collections::VecDeque;
use std::sync::Arc;

use issho::{AccessEvent, IsshoError, SelectionData, SelectionGroupItem};

use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::{Affine, Circle, TrblRectangle};

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use winit::keyboard::KeyCode;

use crate::elements::element_data::ElementData;
use crate::elements::element_id::create_unique_element_id;
use crate::elements::internal_helpers::{apply_generic_container_layout, apply_generic_container_layout_non_dom};
use crate::elements::radiogroup::RadioGroupElement;
use crate::elements::traits::clone_element;
use crate::elements::{
    DynElement, Element, ElementIds, ElementInternals, RadioGroup, RetGuiAccessTree, RetainedElements, scrollable,
};
use crate::events::{Event, EventKind, RadioValueChangedEvent};
use crate::layout::GummyTree;
use crate::style::Unit;
use crate::text::text_context::TextContext;
use crate::{App, auto, px, rgb};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Radio {
    pub(crate) inner: DynElement,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub(crate) struct RadioElement {
    element_data: ElementData,
    circle_layout: ElementData,
    circle: Circle,
    pub(super) value: String,
    label: String,
    hide_radio: bool,
    pub(crate) group: RadioGroup,
}

impl Element for Radio {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::HasElementData for RadioElement {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for RadioElement {
    fn deep_clone(
        &self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
    ) -> DynElement {
        DynElement::new(clone_element::<Self, _>(
            self,
            elements,
            gummy_tree,
            access_tree,
            by_internal_id,
            |element, gummy_tree| {
                let owner_id = element.element_data.internal_id;
                let owner = element.element_data.me;
                let parent = element.element_data.layout.gummy_node_id();
                let circle_node = gummy_tree.clone_node(element.circle_layout.layout.gummy_node_id());
                element.circle_layout.layout.gummy_node_id = Some(circle_node);
                element.circle_layout.internal_id = create_unique_element_id();
                element.circle_layout.me = owner;
                gummy_tree.add_child(parent, circle_node);
                gummy_tree.register_owner(circle_node, owner_id, owner);
                Some(parent)
            },
        ))
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        _text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        apply_generic_container_layout(self, gummy_tree, z_index, scale_factor);
        apply_generic_container_layout_non_dom(&mut self.circle_layout, gummy_tree, z_index, scale_factor);
        let circle_rect = self.circle_layout.layout.local_box_in_parent().content_rectangle();
        self.circle.x = circle_rect.x + self.circle.radius;
        self.circle.y = circle_rect.y + self.circle.radius;
    }

    fn draw(
        &self,
        elements: &RetainedElements,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        _scale_factor: f64,
        _text_context: &mut TextContext,
    ) {
        if !self.is_visible() {
            return;
        }

        self.maybe_start_overlay(renderer);

        self.add_hit_testable(renderer, true, _scale_factor);
        self.draw_borders(renderer, _scale_factor);
        self.maybe_start_layer(renderer, _scale_factor);

        let container_transform = renderer.get_transform();
        let scroll_y = self.element_data.scroll().scroll_y() as f64 * _scale_factor;
        renderer.set_transform(container_transform * Affine::translate((0.0, -scroll_y)));

        if !self.hide_radio {
            if self.is_selected(elements) {
                renderer.draw_circle_outline(
                    self.circle.scale(_scale_factor),
                    Brush::Color(rgb(0, 100, 255)),
                    _scale_factor as f32,
                );
                renderer.draw_circle(
                    self.circle.expand(-4.0).scale(_scale_factor),
                    Brush::Color(rgb(0, 100, 255)),
                );
            } else {
                renderer.draw_circle_outline(
                    self.circle.scale(_scale_factor),
                    Brush::Color(rgb(150, 150, 150)),
                    _scale_factor as f32,
                );
            }
        }

        renderer.set_transform(container_transform);

        self.draw_children(elements, renderer, resource_manager, _scale_factor, _text_context);
        self.maybe_end_layer(renderer);
        self.draw_scrollbar(renderer, _scale_factor);

        self.maybe_end_overlay(renderer);
    }

    fn on_event(
        &mut self,
        elements: &mut RetainedElements,
        _gummy_tree: &mut GummyTree,
        _access_tree: &RetGuiAccessTree,
        _by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        focus_outline_visible: bool,
        _pending_animation_updates: &mut Vec<(DynElement, bool)>,
        event: &mut EventKind,
        _text_context: &mut TextContext,
    ) {
        scrollable::handle_scroll_logic(elements, event_queue, focus, focus_outline_visible, self, event);
        if let EventKind::PointerUp(_) = event {
            self.focus(elements, event_queue, focus, focus_outline_visible);
            self.select(elements, event_queue);
        } else if self.is_focused()
            && let EventKind::KeyDown(keyboard_event) = event
        {
            let handled = match keyboard_event.code {
                KeyCode::Space if !keyboard_event.repeat => {
                    self.select(elements, event_queue);
                    true
                }
                KeyCode::ArrowDown | KeyCode::ArrowRight => {
                    self.move_selection(elements, event_queue, focus, focus_outline_visible, 1)
                }
                KeyCode::ArrowUp | KeyCode::ArrowLeft => {
                    self.move_selection(elements, event_queue, focus, focus_outline_visible, -1)
                }
                _ => false,
            };
            if handled {
                keyboard_event.stop_propagation();
                keyboard_event.prevent_default();
            }
        }
    }

    fn on_access_event(
        &mut self,
        elements: &mut RetainedElements,
        event_queue: &mut VecDeque<EventKind>,
        event: AccessEvent,
    ) -> Result<(), IsshoError> {
        if matches!(event, AccessEvent::Select | AccessEvent::AddToSelection) {
            self.select(elements, event_queue);
        }
        Ok(())
    }
}

impl RadioElement {
    fn is_selected(&self, elements: &RetainedElements) -> bool {
        elements
            .try_get_as::<RadioGroupElement>(self.group.inner)
            .is_some_and(|group| {
                group
                    .selected
                    .is_some_and(|selected| selected.inner == self.element_data.me)
            })
    }

    fn select(&mut self, elements: &mut RetainedElements, event_queue: &mut VecDeque<EventKind>) {
        if self.select_in_group(elements) {
            event_queue.push_back(EventKind::RadioValueChanged(RadioValueChangedEvent::new(
                self.group.inner,
                self.value.clone(),
            )));
        }
    }

    fn select_in_group(&mut self, elements: &mut RetainedElements) -> bool {
        let Some(group) = elements.try_get_as_mut::<RadioGroupElement>(self.group.inner) else {
            return false;
        };
        let me = Radio {
            inner: self.element_data.me,
        };
        if group.selected == Some(me) {
            return false;
        }
        let previous = group.selected.replace(me);
        if let Some(previous) = previous
            && let Some(previous) = elements.try_get_as_mut::<RadioElement>(previous.inner)
        {
            previous.set_accessibility_selection(false);
        }
        self.set_accessibility_selection(true);
        true
    }

    fn move_selection(
        &mut self,
        elements: &mut RetainedElements,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        focus_outline_visible: bool,
        direction: isize,
    ) -> bool {
        let Some(group) = elements.try_get_as::<RadioGroupElement>(self.group.inner) else {
            return false;
        };
        let Some(root) = self.tree_root(elements, self.element_data.me) else {
            return false;
        };
        let members = group
            .members
            .iter()
            .copied()
            .filter(|radio| self.tree_root(elements, radio.inner) == Some(root))
            .collect::<Vec<_>>();
        let Some(current) = members.iter().position(|radio| radio.inner == self.element_data.me) else {
            return false;
        };
        let next_index = if direction < 0 {
            (current + members.len() - 1) % members.len()
        } else {
            (current + 1) % members.len()
        };
        let next = members[next_index];
        if next.inner == self.element_data.me {
            self.select(elements, event_queue);
        } else {
            // This radio is temporarily outside the store while handling its
            // event. Release its focus and update its node before visiting the next.
            self.unfocus(event_queue, focus);
            self.set_accessibility_selection(false);
            elements.dispatch_mut(next.inner, |next, elements| {
                let next = (next as &mut dyn Any).downcast_mut::<RadioElement>().unwrap();
                next.focus(elements, event_queue, focus, focus_outline_visible);
                next.select(elements, event_queue);
            });
        }
        true
    }

    fn tree_root(&self, elements: &RetainedElements, mut element: DynElement) -> Option<DynElement> {
        loop {
            let parent = if element == self.element_data.me {
                self.element_data.parent
            } else {
                elements.try_get(element)?.parent()
            };
            match parent {
                Some(parent) => element = parent,
                None => return Some(element),
            }
        }
    }

    pub(crate) fn set_accessibility_selection(&mut self, selected: bool) {
        self.element_data
            .set_accessibility_selection_data(Some(SelectionData::SelectionGroupItem(SelectionGroupItem {
                is_selected: selected,
            })));
        self.request_window_redraw();
    }

    pub(crate) fn insert(
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        group: RadioGroup,
        value: &str,
        label: &str,
    ) -> DynElement {
        elements.get_as::<RadioGroupElement>(group.inner);
        let radius = 7.0;
        let inner = elements.insert_with(access_tree, by_internal_id, |me, access_tree| {
            Box::new(RadioElement {
                element_data: ElementData::new(me, true, access_tree.clone()),
                circle_layout: ElementData::new_pseudo(me, false, access_tree),
                circle: Circle::new(0.0, 0.0, radius),
                value: value.to_string(),
                label: label.to_string(),
                hide_radio: false,
                group,
            })
        });
        {
            let inner_mut = elements.get_as_mut::<RadioElement>(inner);
            inner_mut.circle_layout.style.set_min_width(Unit::Px(radius * 2.0));
            inner_mut.circle_layout.style.set_min_height(Unit::Px(radius * 2.0));
            inner_mut
                .circle_layout
                .style
                .set_margin(TrblRectangle::new(auto(), px(5), auto(), px(0)));
            inner_mut.element_data.set_accessibility_role(issho::Role::RadioButton);
            inner_mut.element_data.set_accessibility_name(label.to_string());
            inner_mut.set_accessibility_selection(false);
            inner_mut.element_data.create_layout_node(gummy_tree, None);
            inner_mut.circle_layout.create_layout_node(gummy_tree, None);
            let node_id = inner_mut.circle_layout.layout.gummy_node_id();
            gummy_tree.add_child(inner_mut.element_data.layout.gummy_node_id(), node_id);
            gummy_tree.register_owner(node_id, inner_mut.element_data.internal_id, inner);
        }

        elements
            .get_as_mut::<RadioGroupElement>(group.inner)
            .members
            .push(Radio { inner });
        inner
    }
}

impl Radio {
    pub fn new<S: 'static>(app: &mut App<S>, group: RadioGroup, value: &str, label: &str, selected: bool) -> Self {
        let radio = Self {
            inner: RadioElement::insert(
                &mut app.elements,
                &mut app.gummy_tree,
                &app.access_tree,
                &mut app.by_internal_id,
                group,
                value,
                label,
            ),
        };
        if selected {
            app.elements.dispatch_mut(radio.inner, |radio, elements| {
                (radio as &mut dyn Any)
                    .downcast_mut::<RadioElement>()
                    .unwrap()
                    .select_in_group(elements);
            });
        }
        radio
    }

    pub fn is_selected<S: 'static>(&self, app: &App<S>) -> bool {
        app.try_get_as::<RadioElement>(self.inner)
            .is_some_and(|radio| radio.is_selected(&app.elements))
    }

    pub fn select<S: 'static>(&self, app: &mut App<S>) {
        app.elements.try_dispatch_mut(self.inner, |radio, elements| {
            (radio as &mut dyn Any)
                .downcast_mut::<RadioElement>()
                .unwrap()
                .select(elements, &mut app.event_queue);
        });
    }

    /// Hide the default circle radio button.
    pub fn set_hide_radio<S: 'static>(&self, app: &mut App<S>, value: bool) {
        // TODO: Hide in gummy.
        if let Some(inner) = app.try_get_as_mut::<RadioElement>(self.inner) {
            inner.hide_radio = value;
            inner.request_window_redraw();
        }
    }

    /// Hide the default circle radio button.
    pub fn hide_radio<S: 'static>(&self, app: &mut App<S>) {
        self.set_hide_radio(app, true);
    }

    pub fn label<S: 'static>(&self, app: &App<S>) -> String {
        app.try_get_as::<RadioElement>(self.inner)
            .map_or_else(String::new, |radio| radio.label.clone())
    }

    pub fn value<S: 'static>(&self, app: &App<S>) -> String {
        app.try_get_as::<RadioElement>(self.inner)
            .map_or_else(String::new, |radio| radio.value.clone())
    }
}
