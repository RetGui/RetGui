use std::collections::VecDeque;

use retgui_primitives::geometry::Point;

use retgui_renderer::renderer::Renderer;

use rustc_hash::FxHashMap;

use crate::elements::{DynElement, RetainedElements};
use crate::events::pointer_capture::PointerCapture;
use crate::events::{Event, EventCallback, EventCallbackKind, EventKind, PointerId};
use crate::{App, States};

pub(super) fn freeze_target_list(target: DynElement, elements: &RetainedElements) -> VecDeque<DynElement> {
    let mut current = Some(target);
    let mut targets = VecDeque::new();
    while let Some(element) = current {
        targets.push_back(element);
        current = elements.get(element).parent();
    }
    targets
}

pub(super) fn nearest_common_ancestor(a: DynElement, b: DynElement, elements: &RetainedElements) -> Option<DynElement> {
    let a_targets = freeze_target_list(a, elements);
    freeze_target_list(b, elements)
        .into_iter()
        .find(|candidate| a_targets.contains(candidate))
}

pub(super) struct TargetSearchContext<'a> {
    pub renderer: &'a dyn Renderer,
    pub target_scratch: &'a mut Vec<DynElement>,
    pub pointer_capture: &'a PointerCapture,
    pub elements: &'a RetainedElements,
    pub by_internal_id: &'a FxHashMap<u64, DynElement>,
}

pub(super) fn find_target(
    root: DynElement,
    mouse_position: Option<Point>,
    message: &EventKind,
    pointer_id: &PointerId,
    context: TargetSearchContext<'_>,
) -> DynElement {
    let TargetSearchContext {
        renderer,
        target_scratch,
        pointer_capture,
        elements,
        by_internal_id,
    } = context;

    if let Some(target) = pointer_capture.find_pointer_capture_target(message, pointer_id) {
        return target;
    }

    let physical_mouse_position = mouse_position.map(|point| {
        let scale = elements.get(root).element_data().applied_scale_factor;
        Point::new(point.x * scale, point.y * scale)
    });

    let targets = &renderer.render_list().targets;
    target_scratch.extend(targets.iter().rev().filter_map(|item| {
        physical_mouse_position
            .is_some_and(|point| item.rectangle.contains(&point))
            .then(|| {
                by_internal_id
                    .get(&item.custom_id)
                    .copied()
                    .filter(|element| elements.contains(*element))
            })
            .flatten()
    }));

    target_scratch
        .drain(..)
        .find(|element| mouse_position.is_some_and(|point| elements.get(*element).in_bounds(point)))
        .unwrap_or(root)
}

pub(super) fn call_user_event_handlers(event: &mut EventKind, capturing: bool, app: &mut App, states: &mut States) {
    let Some(current_target) = app.elements.try_get(event.current_target()) else {
        return;
    };
    let callbacks = current_target.element_data().event_callbacks.clone();

    for EventCallback {
        callback,
        capturing: callback_capturing,
    } in callbacks
    {
        if callback_capturing != capturing {
            continue;
        }
        match (&mut *event, callback) {
            (EventKind::PointerEnter(event), EventCallbackKind::PointerEnter(handler)) => handler(event, app, states),
            (EventKind::PointerLeave(event), EventCallbackKind::PointerLeave(handler)) => handler(event, app, states),
            (EventKind::Click(event), EventCallbackKind::Click(handler)) => handler(event, app, states),
            (EventKind::Custom(event), EventCallbackKind::Custom(handler)) => handler(event, app, states),
            (EventKind::Focus(event), EventCallbackKind::Focus(handler)) => handler(event, app, states),
            (EventKind::GotPointerCapture(event), EventCallbackKind::GotPointerCapture(handler))
            | (EventKind::LostPointerCapture(event), EventCallbackKind::LostPointerCapture(handler)) => {
                handler(event, app, states)
            }
            (EventKind::Scroll(event), EventCallbackKind::Scroll(handler)) => handler(event, app, states),
            (EventKind::Unfocus(event), EventCallbackKind::Unfocus(handler)) => handler(event, app, states),
            (EventKind::PointerUp(event), EventCallbackKind::PointerButtonUp(handler))
            | (EventKind::PointerDown(event), EventCallbackKind::PointerButtonDown(handler)) => {
                handler(event, app, states)
            }
            (EventKind::KeyDown(event), EventCallbackKind::KeyboardInput(handler))
            | (EventKind::KeyUp(event), EventCallbackKind::KeyboardInput(handler)) => handler(event, app, states),
            (EventKind::PointerMoved(event), EventCallbackKind::PointerMoved(handler)) => handler(event, app, states),
            (EventKind::DropdownItemSelected(event), EventCallbackKind::DropdownItemSelected(handler)) => {
                handler(event, app, states)
            }
            (EventKind::SliderValueChanged(event), EventCallbackKind::SliderValueChanged(handler)) => {
                handler(event, app, states)
            }
            (EventKind::RadioValueChanged(event), EventCallbackKind::RadioValueChanged(handler)) => {
                handler(event, app, states)
            }
            (EventKind::CheckboxToggled(event), EventCallbackKind::CheckboxToggled(handler)) => {
                handler(event, app, states)
            }
            (EventKind::TextInputChanged(event), EventCallbackKind::TextInputChanged(handler)) => {
                handler(event, app, states)
            }
            _ => {}
        }
    }
}
