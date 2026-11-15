//! A selectable circle.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use issho::{AccessEvent, IsshoError, SelectionData, SelectionGroupItem};

use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::{Affine, Circle, TrblRectangle};


use retgui_resource_manager::ResourceManager;

use winit::keyboard::KeyCode;
use retgui_renderer::RenderList;
use crate::app::{GUMMY_TREE, queue_event};
use crate::elements::element_data::ElementData;
use crate::elements::element_id::create_unique_element_id;
use crate::elements::internal_helpers::{apply_generic_container_layout, apply_generic_container_layout_non_dom, push_child_to_element};
use crate::elements::traits::clone_element;
use crate::elements::{AsElement, DynElement, Element, ElementInternals, scrollable};
use crate::events::{Event, EventKind, RadioValueChangedEvent};
use crate::layout::GummyTree;
use crate::style::Unit;
use crate::text::text_context::TextContext;
use crate::{auto, px, rgb};

#[derive(Default)]
struct RadioSelection {
    selected: Option<Weak<RefCell<dyn ElementInternals>>>,
}

thread_local! {
    static RADIO_SELECTIONS: RefCell<std::collections::HashMap<usize, Weak<RefCell<RadioSelection>>>> = RefCell::new(Default::default());
}

fn selection_for(value: &Rc<RefCell<String>>) -> Rc<RefCell<RadioSelection>> {
    RADIO_SELECTIONS.with_borrow_mut(|selections| {
        let key = Rc::as_ptr(value) as usize;
        if let Some(selection) = selections.get(&key).and_then(Weak::upgrade) {
            return selection;
        }
        selections.retain(|_, selection| selection.strong_count() > 0);
        let selection = Rc::new(RefCell::new(RadioSelection::default()));
        selections.insert(key, Rc::downgrade(&selection));
        selection
    })
}

#[derive(Clone)]
pub struct Radio {
    pub(crate) inner: Rc<RefCell<RadioInner>>,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub(crate) struct RadioInner {
    element_data: ElementData,
    circle_layout: ElementData,
    circle: Circle,
    value: String,
    label: String,
    hide_radio: bool,
    active_value: Rc<RefCell<String>>,
    selection: Rc<RefCell<RadioSelection>>,
}

impl Default for Radio {
    fn default() -> Self {
        Self::new("", "radio item", Rc::new(RefCell::new("".to_string())))
    }
}

impl Element for Radio {}

impl Drop for RadioInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for Radio {
    fn with<R>(&self, callback: impl FnOnce(&dyn ElementInternals) -> R) -> R {
        callback(&*self.inner.borrow())
    }

    fn with_mut<R>(&self, callback: impl FnOnce(&mut dyn ElementInternals) -> R) -> R {
        callback(&mut *self.inner.borrow_mut())
    }
}

impl crate::elements::ElementData for RadioInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for RadioInner {
    fn deep_clone(&self) -> DynElement {
        DynElement::new(clone_element::<Self, _>(self, |element, gummy_tree| {
            let mut element = element.borrow_mut();
            let owner_id = element.element_data.internal_id;
            let owner = element.element_data.me.clone();
            let parent = element.element_data.layout.gummy_node_id();
            let circle_node = gummy_tree.clone_node(element.circle_layout.layout.gummy_node_id());
            element.circle_layout.layout.gummy_node_id = Some(circle_node);
            element.circle_layout.internal_id = create_unique_element_id();
            element.circle_layout.me = owner.clone();
            gummy_tree.add_child(parent, circle_node);
            gummy_tree.register_owner(circle_node, owner_id, owner);
            Some(parent)
        }))
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
        &mut self,
        render_list: &mut RenderList,
        resource_manager: Arc<ResourceManager>,
        _scale_factor: f64,
        _text_context: &mut TextContext,
    ) {
        if !self.is_visible() {
            return;
        }

        self.maybe_start_overlay(render_list);

        self.add_hit_testable(render_list, true, _scale_factor);
        self.draw_borders(render_list, _scale_factor);
        self.maybe_start_layer(render_list, _scale_factor);

        let container_transform = render_list.get_transform();
        let scroll_y = self.element_data.scroll().scroll_y() as f64 * _scale_factor;
        render_list.set_transform(container_transform * Affine::translate((0.0, -scroll_y)));

        if !self.hide_radio {
            if self.is_selected() {
                render_list.draw_circle_outline(
                    self.circle.scale(_scale_factor),
                    Brush::Color(rgb(0, 100, 255)),
                    _scale_factor as f32,
                );
                render_list.draw_circle(
                    self.circle.expand(-4.0).scale(_scale_factor),
                    Brush::Color(rgb(0, 100, 255)),
                );
            } else {
                render_list.draw_circle_outline(
                    self.circle.scale(_scale_factor),
                    Brush::Color(rgb(150, 150, 150)),
                    _scale_factor as f32,
                );
            }
        }

        render_list.set_transform(container_transform);

        self.draw_children(render_list, resource_manager, _scale_factor, _text_context);
        self.maybe_end_layer(render_list);
        self.draw_scrollbar(render_list, _scale_factor);

        self.maybe_end_overlay(render_list);
    }

    fn on_event(&mut self, event: &mut EventKind, _text_context: &mut TextContext) {
        scrollable::handle_scroll_logic(self, event);
        if let EventKind::PointerUp(_) = event {
            self.focus();
            self.set_value();
        } else if self.is_focused()
            && let EventKind::KeyDown(keyboard_event) = event
            && keyboard_event.code == KeyCode::Space
            && !keyboard_event.repeat
        {
            self.set_value();
            keyboard_event.stop_propagation();
            keyboard_event.prevent_default();
        }
    }

    fn push(&mut self, child: DynElement) {
        push_child_to_element(self, child.inner);
    }

    fn on_access_event(&mut self, event: AccessEvent) -> Result<(), IsshoError> {
        if matches!(event, AccessEvent::Select | AccessEvent::AddToSelection) && !self.is_selected() {
            self.set_value();
        }
        Ok(())
    }
}

impl RadioInner {
    fn set_value(&mut self) {
        self.set_value_from_group();
    }

    pub(super) fn set_value_from_group(&mut self) {
        if self.selection.borrow().selected.as_ref().is_some_and(|selected| selected.ptr_eq(&self.element_data.me)) {
            return;
        }
        let previous = self.selection.borrow_mut().selected.clone();
        self.selection.borrow_mut().selected = Some(self.element_data.me.clone());
        self.active_value.replace(self.value.clone());
        if let Some(previous) = previous.and_then(|previous| previous.upgrade()) {
            let mut previous = previous.borrow_mut();
            previous.element_data_mut().set_accessibility_selection_data(Some(
                SelectionData::SelectionGroupItem(SelectionGroupItem { is_selected: false }),
            ));
            previous.request_window_redraw();
        }
        self.set_accessibility_selection();
        let target = self.element_data.me.upgrade().unwrap();
        queue_event(EventKind::RadioValueChanged(RadioValueChangedEvent::new(
            DynElement::new(target),
            self.active_value.clone(),
        )));
    }

    fn is_selected(&self) -> bool {
        self.active_value.borrow().as_str() == self.value
    }

    pub(super) fn set_accessibility_selection(&mut self) {
        let is_selected = self.is_selected();
        self.element_data
            .set_accessibility_selection_data(Some(SelectionData::SelectionGroupItem(SelectionGroupItem {
                is_selected,
            })));
        self.request_window_redraw();
    }
}

impl Radio {
    pub fn new(value: &str, label: &str, active_value: Rc<RefCell<String>>) -> Self {
        let selection = selection_for(&active_value);
        let radius = 7.0;
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<RadioInner>>| {
            RefCell::new(RadioInner {
                element_data: ElementData::new(me.clone(), true),
                circle_layout: ElementData::new_pseudo(me.clone(), false),
                circle: Circle::new(0.0, 0.0, radius),
                value: value.to_string(),
                label: label.to_string(),
                hide_radio: false,
                active_value,
                selection,
            })
        });
        let mut inner_mut = inner.borrow_mut();
        inner_mut.element_data.create_layout_node(None);

        inner_mut.circle_layout.style.set_min_width(Unit::Px(radius * 2.0));
        inner_mut.circle_layout.style.set_min_height(Unit::Px(radius * 2.0));
        inner_mut
            .circle_layout
            .style
            .set_margin(TrblRectangle::new(auto(), px(5), auto(), px(0)));
        inner_mut.circle_layout.create_layout_node(None);
        GUMMY_TREE.with_borrow_mut(|gummy_tree| {
            let node_id = inner_mut.circle_layout.layout.gummy_node_id();
            gummy_tree.add_child(inner_mut.element_data.layout.gummy_node_id(), node_id);
            let owner: Rc<RefCell<dyn ElementInternals>> = inner.clone();
            gummy_tree.register_owner(node_id, inner_mut.element_data.internal_id, Rc::downgrade(&owner));
        });
        {
            inner_mut.element_data.set_accessibility_role(issho::Role::RadioButton);
            inner_mut.element_data.set_accessibility_name(label.to_string());
            if inner_mut.is_selected() {
                inner_mut.selection.borrow_mut().selected = Some(inner_mut.element_data.me.clone());
            }
            inner_mut.set_accessibility_selection();
        }

        drop(inner_mut);
        Self { inner }
    }

    /// Hide the default circle radio button.
    pub fn set_hide_radio(&mut self, value: bool) {
        // TODO: Hide in gummy.
        let mut inner = self.inner.borrow_mut();
        inner.hide_radio = value;
        inner.request_window_redraw();
    }

    /// Hide the default circle radio button.
    pub fn hide_radio(mut self) -> Self {
        self.set_hide_radio(true);
        self
    }

    pub fn get_label(&self) -> String {
        self.inner.borrow().label.clone()
    }

    pub fn get_value(&self) -> String {
        self.inner.borrow().value.clone()
    }
}

#[cfg(test)]
mod retained_tests {
    use super::*;
    use crate::elements::Container;

    #[test]
    fn selection_changes_only_previous_and_next_radio() {
        let value = Rc::new(RefCell::new("first".to_string()));
        let first = Radio::new("first", "First", value.clone());
        let second = Radio::new("second", "Second", value);
        let unrelated = Container::new();
        let _root = Container::new().push(first.clone()).push(second.clone()).push(unrelated.clone());
        // An event must not borrow or traverse unrelated elements.
        let _unrelated_borrow = unrelated.inner.borrow_mut();
        assert!(second.inner.borrow_mut().on_access_event(AccessEvent::Select).is_ok());
        for (radio, selected) in [(&first, false), (&second, true)] {
            radio.with(|element| {
                let data = element.element_data();
                let node = data.access_tree.get_node(data.access_key.unwrap()).unwrap();
                let Some(SelectionData::SelectionGroupItem(item)) = node.selection_data() else {
                    panic!("missing radio selection");
                };
                assert_eq!(item.is_selected, selected);
            });
        }
    }
}
