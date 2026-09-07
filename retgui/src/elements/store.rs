use std::any::Any;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::mem;
use std::sync::atomic::{AtomicU64, Ordering};

use rustc_hash::{FxHashMap, FxHashSet};

use slotmap::{DefaultKey, Key, SlotMap};

use crate::accessibility::RetGuiAccessTree;
use crate::elements::radiogroup::RadioGroupElement;
use crate::elements::traits::finish_clone;
use crate::elements::{DynElement, ElementIds, ElementInternals, Radio, RadioElement, WindowElement};
use crate::events::EventKind;
use crate::layout::GummyTree;

static NEXT_STORE_ID: AtomicU64 = AtomicU64::new(1);

pub struct RetainedElements {
    id: u64,
    slots: SlotMap<DefaultKey, Option<Box<dyn ElementInternals>>>,
}

impl RetainedElements {
    pub fn get(&self, element: DynElement) -> &dyn ElementInternals {
        assert_eq!(
            element.store_id(),
            self.id,
            "element handle belongs to a different store"
        );
        self.slots
            .get(element.key())
            .and_then(Option::as_deref)
            .expect("element handle no longer belongs to this store")
    }

    pub fn contains(&self, element: DynElement) -> bool {
        element.store_id() == self.id && self.slots.get(element.key()).is_some_and(Option::is_some)
    }

    pub fn get_mut(&mut self, element: DynElement) -> &mut dyn ElementInternals {
        assert_eq!(
            element.store_id(),
            self.id,
            "element handle belongs to a different store"
        );
        self.slots
            .get_mut(element.key())
            .and_then(Option::as_deref_mut)
            .expect("element handle no longer belongs to this store")
    }

    pub fn try_get_mut(&mut self, element: DynElement) -> Option<&mut dyn ElementInternals> {
        if element.store_id() != self.id {
            return None;
        }
        self.slots.get_mut(element.key()).and_then(Option::as_deref_mut)
    }

    pub fn get_as_mut<T: ElementInternals>(&mut self, element: DynElement) -> &mut T {
        (self.get_mut(element) as &mut dyn Any)
            .downcast_mut()
            .expect("typed element handle changed type")
    }

    pub fn try_get_as_mut<T: ElementInternals>(&mut self, element: DynElement) -> Option<&mut T> {
        Some(
            (self.try_get_mut(element)? as &mut dyn Any)
                .downcast_mut()
                .expect("typed element handle changed type"),
        )
    }

    pub(crate) fn new() -> Self {
        Self {
            id: NEXT_STORE_ID.fetch_add(1, Ordering::Relaxed),
            slots: SlotMap::with_key(),
        }
    }

    pub fn insert_with(
        &mut self,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut FxHashMap<u64, DynElement>,
        create: impl FnOnce(DynElement, RetGuiAccessTree) -> Box<dyn ElementInternals>,
    ) -> DynElement {
        let access_tree = access_tree.clone();
        let store_id = self.id;
        let key = self
            .slots
            .insert_with_key(|key| Some(create(DynElement::from_key(key, store_id), access_tree)));
        let handle = DynElement::from_key(key, store_id);
        let id = self.get(handle).element_data().internal_id;
        by_internal_id.insert(id, handle);
        handle
    }

    pub fn try_get(&self, element: DynElement) -> Option<&dyn ElementInternals> {
        if element.store_id() != self.id {
            return None;
        }
        self.slots.get(element.key()).and_then(Option::as_deref)
    }

    pub fn get_for_draw(&self, element: DynElement) -> &dyn ElementInternals {
        debug_assert_eq!(element.store_id(), self.id);
        self.slots[element.key()]
            .as_deref()
            .expect("retained tree contains a deleted element")
    }

    pub fn get_as<T: ElementInternals>(&self, element: DynElement) -> &T {
        (self.get(element) as &dyn Any)
            .downcast_ref()
            .expect("typed element handle changed type")
    }

    pub fn try_get_as<T: ElementInternals>(&self, element: DynElement) -> Option<&T> {
        Some(
            (self.try_get(element)? as &dyn Any)
                .downcast_ref()
                .expect("typed element handle changed type"),
        )
    }

    pub fn deep_clone(
        &mut self,
        source: DynElement,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
    ) -> DynElement {
        let cloned = self.dispatch_mut(source, |source, elements| {
            source.deep_clone(elements, gummy_tree, access_tree, by_internal_id)
        });
        finish_clone(self, source, cloned, gummy_tree, access_tree, by_internal_id);
        cloned
    }

    pub(crate) fn delete_all_children(
        &mut self,
        gummy_tree: &mut GummyTree,
        by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        parent: DynElement,
    ) {
        let roots = mem::take(&mut self.get_mut(parent).element_data_mut().children);
        if roots.is_empty() {
            return;
        }

        for root in &roots {
            self.get_mut(*root).element_data_mut().parent = None;
        }

        let mut subtree = Vec::new();
        let mut seen = FxHashSet::default();
        let mut pending = roots.clone();
        while let Some(element) = pending.pop() {
            if !seen.insert(element) {
                continue;
            }
            pending.extend(self.get(element).element_data().children.iter().copied());
            subtree.push(element);
        }

        if let Some(focused) = *focus
            && seen.contains(&focused)
        {
            self.get_mut(focused).unfocus(event_queue, focus);
        }

        if let Some(window) = self.get(parent).element_data().window {
            let capture = &mut self.get_as_mut::<WindowElement>(window).pointer_capture;
            for element in &subtree {
                capture.remove_element(*element);
            }
        }

        let layout_parent = self.get(parent).child_layout_parent();
        let layout_roots = roots
            .iter()
            .filter_map(|root| self.get(*root).element_data().layout.gummy_node_id)
            .collect::<Vec<_>>();
        if let Some(parent) = layout_parent {
            gummy_tree.set_children(parent, &[]);
        }
        for root in layout_roots {
            gummy_tree.remove_subtree(root);
        }

        let parent_data = self.get(parent).element_data();
        if let Some(parent_key) = parent_data.access_key {
            parent_data.access_tree.set_children(parent_key, &[]);
        }

        for &handle in &subtree {
            if let Some(radio) = (self.get(handle) as &dyn Any).downcast_ref::<RadioElement>() {
                let group = radio.group;
                if let Some(group) = self.try_get_as_mut::<RadioGroupElement>(group.inner) {
                    group.members.retain(|member| member.inner != handle);
                    if group.selected == Some(Radio { inner: handle }) {
                        group.selected = None;
                    }
                }
            } else if let Some(group) = (self.get(handle) as &dyn Any).downcast_ref::<RadioGroupElement>() {
                let members = group.members.clone();
                for member in members {
                    if !seen.contains(&member.inner)
                        && let Some(radio) = self.try_get_as_mut::<RadioElement>(member.inner)
                    {
                        radio.set_accessibility_selection(false);
                    }
                }
            }
        }

        for handle in subtree.into_iter().rev() {
            if let Some(element) = self.slots.remove(handle.key()).flatten() {
                by_internal_id.remove(&element.element_data().internal_id);
            }
        }

        self.get(parent).request_window_redraw();
    }

    pub fn dispatch_mut<R>(
        &mut self,
        handle: DynElement,
        callback: impl FnOnce(&mut dyn ElementInternals, &mut Self) -> R,
    ) -> R {
        assert_eq!(
            handle.store_id(),
            self.id,
            "element handle belongs to a different store"
        );
        let mut element = self.slots[handle.key()]
            .take()
            .expect("element is already being visited");
        let result = callback(element.as_mut(), self);
        self.slots[handle.key()] = Some(element);
        result
    }

    pub fn try_dispatch_mut<R>(
        &mut self,
        handle: DynElement,
        callback: impl FnOnce(&mut dyn ElementInternals, &mut Self) -> R,
    ) -> Option<R> {
        if handle.store_id() != self.id {
            return None;
        }
        let mut element = self.slots.get_mut(handle.key())?.take()?;
        let result = callback(element.as_mut(), self);
        if let Some(slot) = self.slots.get_mut(handle.key()) {
            *slot = Some(element);
        }
        Some(result)
    }

    pub fn store_id(&self) -> u64 {
        self.id
    }
}

pub struct States {
    id: u64,
    slots: SlotMap<DefaultKey, Box<dyn Any>>,
}

impl Default for States {
    fn default() -> Self {
        Self::new()
    }
}

impl States {
    pub fn new() -> Self {
        Self {
            id: NEXT_STORE_ID.fetch_add(1, Ordering::Relaxed),
            slots: SlotMap::with_key(),
        }
    }

    pub fn insert<T: 'static>(&mut self, value: T) -> State<T> {
        State {
            key: self.slots.insert(Box::new(value)),
            store_id: self.id,
            marker: PhantomData,
        }
    }

    pub fn insert_with<T: 'static>(&mut self, create: impl FnOnce(State<T>) -> T) -> State<T> {
        let store_id = self.id;
        let key = self.slots.insert_with_key(|key| {
            Box::new(create(State {
                key,
                store_id,
                marker: PhantomData,
            }))
        });
        State {
            key,
            store_id,
            marker: PhantomData,
        }
    }

    pub fn get<T: 'static>(&self, state: State<T>) -> &T {
        assert_eq!(state.store_id, self.id, "state handle belongs to a different store");
        self.slots[state.key].downcast_ref().expect("state handle changed type")
    }

    pub fn get_mut<T: 'static>(&mut self, state: State<T>) -> &mut T {
        assert_eq!(state.store_id, self.id, "state handle belongs to a different store");
        self.slots[state.key].downcast_mut().expect("state handle changed type")
    }
}

pub struct State<T> {
    key: DefaultKey,
    store_id: u64,
    marker: PhantomData<fn() -> T>,
}

impl<T> Copy for State<T> {}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for State<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.store_id == other.store_id
    }
}

impl<T> Eq for State<T> {}

impl<T> std::fmt::Debug for State<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("State")
            .field(&(self.store_id, self.key.data()))
            .finish()
    }
}

impl<T: 'static> State<T> {
    pub fn borrow(self, states: &States) -> &T {
        states.get(self)
    }

    pub fn borrow_mut(self, states: &mut States) -> &mut T {
        states.get_mut(self)
    }

    pub fn with_mut<R>(self, states: &mut States, callback: impl FnOnce(&mut T) -> R) -> R {
        callback(states.get_mut(self))
    }
}
