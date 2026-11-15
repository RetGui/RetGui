//! Stores one or more elements.

use std::any::Any;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use issho::{SelectionData, SelectionGroup};

use retgui_renderer::RenderList;

use retgui_resource_manager::ResourceManager;

use winit::keyboard::{KeyCode, ModifiersState};

use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container, push_child_to_element};
use crate::elements::traits::clone_element;
use crate::elements::{AsElement, DynElement, Element, ElementInternals, RadioInner, scrollable};
use crate::events::{Event, EventKind};
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;

#[derive(Clone)]
pub struct RadioGroup {
    pub(crate) inner: Rc<RefCell<RadioGroupInner>>,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub(crate) struct RadioGroupInner {
    element_data: ElementData,
}

impl Default for RadioGroup {
    fn default() -> Self {
        Self::new("Radio Group")
    }
}

impl Element for RadioGroup {}

impl Drop for RadioGroupInner {
    fn drop(&mut self) {
        ElementInternals::drop(self)
    }
}

impl AsElement for RadioGroup {
    fn with<R>(&self, callback: impl FnOnce(&dyn ElementInternals) -> R) -> R {
        callback(&*self.inner.borrow())
    }

    fn with_mut<R>(&self, callback: impl FnOnce(&mut dyn ElementInternals) -> R) -> R {
        callback(&mut *self.inner.borrow_mut())
    }
}

impl crate::elements::ElementData for RadioGroupInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for RadioGroupInner {
    fn deep_clone(&self) -> DynElement {
        DynElement::new(clone_element::<Self, _>(self, |_, _| None))
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

        if let EventKind::KeyDown(keyboard_event) = event {
            let direction = selection_direction(keyboard_event.code, keyboard_event.modifiers);

            if direction.is_some_and(|direction| self.move_selection(direction)) {
                keyboard_event.stop_propagation();
                keyboard_event.prevent_default();
            }
        }
    }

    fn push(&mut self, child: DynElement) {
        push_child_to_element(self, child.inner);
    }
}

fn selection_direction(code: KeyCode, modifiers: ModifiersState) -> Option<isize> {
    if modifiers.control_key() || modifiers.alt_key() || modifiers.meta_key() {
        return None;
    }
    match code {
        KeyCode::ArrowDown | KeyCode::ArrowRight => Some(1),
        KeyCode::ArrowUp | KeyCode::ArrowLeft => Some(-1),
        _ => None,
    }
}

impl RadioGroupInner {
    fn move_selection(&mut self, direction: isize) -> bool {
        fn collect(children: &[DynElement], radios: &mut Vec<DynElement>) {
            for child in children {
                let element = child.borrow();
                if (element.deref() as &dyn Any).is::<RadioInner>() {
                    radios.push(child.clone());
                } else if !(element.deref() as &dyn Any).is::<RadioGroupInner>() {
                    collect(&element.element_data().children, radios);
                }
            }
        }
        let mut radios = Vec::new();
        collect(&self.element_data.children, &mut radios);
        let Some(current_index) = radios.iter().position(|radio| radio.borrow().is_focused()) else {
            return false;
        };

        let next_index = if direction < 0 {
            (current_index + radios.len() - 1) % radios.len()
        } else {
            (current_index + 1) % radios.len()
        };
        {
            let mut next = radios[next_index].inner.borrow_mut();
            let next = (next.deref_mut() as &mut dyn Any)
                .downcast_mut::<RadioInner>()
                .expect("radio group child changed type during keyboard navigation");
            next.focus();
            next.set_value_from_group();
        }
        true
    }
}

impl RadioGroup {
    pub fn new(label: &str) -> Self {
        let inner = Rc::new_cyclic(|me: &Weak<RefCell<RadioGroupInner>>| {
            RefCell::new(RadioGroupInner {
                element_data: ElementData::new(me.clone(), true),
            })
        });
        let mut inner_mut = inner.borrow_mut();
        inner_mut.element_data.create_layout_node(None);
        {
            inner_mut.element_data.set_accessibility_role(issho::Role::Group);
            inner_mut.element_data.set_accessibility_name(label.to_string());
            inner_mut
                .element_data
                .set_accessibility_selection_data(Some(SelectionData::SelectionGroup(SelectionGroup {
                    is_mandatory: true,
                    multiple_selectable: false,
                })));
        }
        drop(inner_mut);
        Self { inner }
    }
}
