//! A group of radios.

use std::collections::VecDeque;
use std::sync::Arc;

use issho::{SelectionData, SelectionGroup};

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use crate::App;
use crate::elements::element_data::ElementData;
use crate::elements::internal_helpers::{apply_generic_container_layout, draw_generic_container};
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementIds, ElementInternals, Radio, RadioElement, RetGuiAccessTree, RetainedElements, scrollable};
use crate::events::EventKind;
use crate::layout::GummyTree;
use crate::text::text_context::TextContext;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RadioGroup {
    pub(crate) inner: DynElement,
}

/// Stores one or more elements.
///
/// If overflow is set to scroll, it will become scrollable.
#[derive(Clone)]
pub(crate) struct RadioGroupElement {
    element_data: ElementData,
    pub(crate) selected: Option<Radio>,
    pub(crate) members: Vec<Radio>,
}

impl Element for RadioGroup {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::HasElementData for RadioGroupElement {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for RadioGroupElement {
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
            |_, _| None,
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
    }

    fn draw(
        &self,
        elements: &RetainedElements,
        renderer: &mut dyn Renderer,
        resource_manager: Arc<ResourceManager>,
        scale_factor: f64,
        text_context: &mut TextContext,
    ) {
        draw_generic_container(self, elements, renderer, resource_manager, text_context, scale_factor);
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
    }
}

impl RadioGroupElement {
    pub(crate) fn insert(
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        label: &str,
    ) -> DynElement {
        let inner = elements.insert_with(access_tree, by_internal_id, |me, access_tree| {
            Box::new(RadioGroupElement {
                element_data: ElementData::new(me, true, access_tree),
                selected: None,
                members: Vec::new(),
            })
        });
        let inner_mut = elements.get_as_mut::<RadioGroupElement>(inner);
        inner_mut.element_data.create_layout_node(gummy_tree, None);
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
        inner
    }
}

impl RadioGroup {
    pub fn new<S: 'static>(app: &mut App<S>, label: &str) -> Self {
        Self {
            inner: RadioGroupElement::insert(
                &mut app.elements,
                &mut app.gummy_tree,
                &app.access_tree,
                &mut app.by_internal_id,
                label,
            ),
        }
    }

    pub fn value<S: 'static>(&self, app: &App<S>) -> Option<String> {
        let selected = app.try_get_as::<RadioGroupElement>(self.inner)?.selected?;
        Some(app.try_get_as::<RadioElement>(selected.inner)?.value.clone())
    }

    pub fn set_value<S: 'static>(&self, app: &mut App<S>, value: &str) -> bool {
        let Some(group) = app.try_get_as::<RadioGroupElement>(self.inner) else {
            return false;
        };
        let selected = group.members.iter().copied().find(|member| {
            app.try_get_as::<RadioElement>(member.inner)
                .is_some_and(|radio| radio.value == value)
        });
        if let Some(radio) = selected {
            radio.select(app);
            true
        } else {
            false
        }
    }
}
