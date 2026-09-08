use std::any::Any;

use rustc_hash::FxHashMap;

use crate::elements::element_id::create_unique_element_id;
use crate::elements::radiogroup::RadioGroupElement;
use crate::elements::{DropdownElement, DynElement, ElementIds, ElementInternals, Radio, RadioElement, RadioGroup, RetGuiAccessTree, RetainedElements};
use crate::layout::GummyTree;

pub fn clone_element<T, F>(
    source: &T,
    elements: &mut RetainedElements,
    gummy_tree: &mut crate::layout::GummyTree,
    access_tree: &RetGuiAccessTree,
    by_internal_id: &mut ElementIds,
    remap: F,
) -> DynElement
where
    T: ElementInternals + Clone + 'static,
    F: FnOnce(&mut T, &mut crate::layout::GummyTree) -> Option<gummy::NodeId>,
{
    let source_children = source.element_data().children.clone();
    let mut clone = source.clone();
    let new_element = elements.insert_with(access_tree, by_internal_id, |me, tree| {
        let data = clone.element_data_mut();
        data.internal_id = create_unique_element_id();
        data.me = me;
        data.parent = None;
        if let Some(color) = data.unfocused_outline_color.take() {
            data.style.set_outline_color(color);
        }
        if let Some(width) = data.unfocused_outline_width.take() {
            data.style.set_outline_width(width);
        }

        let source_key = data.access_key.expect("source accessibility node was not created");
        let mut node = data
            .access_tree
            .get_node(source_key)
            .expect("source accessibility node was not created")
            .clone();
        node.set_context(me);
        let key = tree.insert_node(node, None);
        data.access_tree = tree;
        data.access_key = Some(key);
        data.access_root = Some(key);
        Box::new(clone)
    });

    let (cloned_access_tree, access_key, access_scale_factor, node_id) = {
        let tree = &mut *gummy_tree;
        let element = elements.get_as_mut::<T>(new_element);
        let data = element.element_data_mut();
        let node_id = data.layout.gummy_node_id_mut();
        *node_id = tree.clone_node(*node_id);
        tree.register_owner(*node_id, data.internal_id, new_element);
        tree.mark_dirty(*node_id);
        (
            data.access_tree.clone(),
            data.access_key.unwrap(),
            data.access_scale_factor.get(),
            *node_id,
        )
    };

    let child_layout_parent = {
        let tree = &mut *gummy_tree;
        remap(elements.get_as_mut::<T>(new_element), tree).unwrap_or(node_id)
    };

    let mut children = Vec::with_capacity(source_children.len());
    for child in source_children {
        let cloned_child = elements.dispatch_mut(child, |child, elements| {
            child.deep_clone(elements, gummy_tree, access_tree, by_internal_id)
        });
        elements.get_mut(cloned_child).element_data_mut().parent = Some(new_element);
        elements.dispatch_mut(cloned_child, |child, elements| {
            crate::accessibility::reparent_subtree(
                elements,
                child,
                &cloned_access_tree,
                access_key,
                access_key,
                access_scale_factor,
            )
        });
        let child_node = elements.get(cloned_child).element_data().layout.gummy_node_id.unwrap();
        gummy_tree.add_child(child_layout_parent, child_node);
        children.push(cloned_child);
    }
    elements.get_mut(new_element).element_data_mut().children = children;
    gummy_tree.request_apply_layout(node_id);
    new_element
}

pub(crate) fn subtree_clone_pairs(
    elements: &RetainedElements,
    source: DynElement,
    cloned_root: DynElement,
) -> Vec<(DynElement, DynElement)> {
    let mut pairs = vec![(source, cloned_root)];
    let mut index = 0;
    while let Some(&(original, cloned)) = pairs.get(index) {
        let original_children = &elements.get(original).element_data().children;
        let cloned_children = &elements.get(cloned).element_data().children;
        pairs.extend(original_children.iter().copied().zip(cloned_children.iter().copied()));
        index += 1;
    }
    pairs
}

pub(crate) fn finish_clone(
    elements: &mut RetainedElements,
    source: DynElement,
    cloned_root: DynElement,
    gummy_tree: &mut GummyTree,
    access_tree: &RetGuiAccessTree,
    by_internal_id: &mut ElementIds,
) {
    let pairs = subtree_clone_pairs(elements, source, cloned_root);
    let cloned_by_source: FxHashMap<_, _> = pairs.iter().copied().collect();

    // All children are restored to the store now, so groups in a different
    // branch can be remapped without changing any original group's members.
    for &(original, cloned) in &pairs {
        let Some(group) = (elements.get(original) as &dyn Any).downcast_ref::<RadioGroupElement>() else {
            continue;
        };
        let members = group
            .members
            .iter()
            .filter_map(|member| cloned_by_source.get(&member.inner).map(|&inner| Radio { inner }))
            .collect();
        let selected = group
            .selected
            .and_then(|selected| cloned_by_source.get(&selected.inner).map(|&inner| Radio { inner }));
        let group = elements.get_as_mut::<RadioGroupElement>(cloned);
        group.members = members;
        group.selected = selected;
    }

    for &(_, cloned) in &pairs {
        let Some(radio) = (elements.get(cloned) as &dyn Any).downcast_ref::<RadioElement>() else {
            continue;
        };
        let mut group = radio.group;
        if let Some(&inner) = cloned_by_source.get(&group.inner) {
            group = RadioGroup { inner };
        } else if let Some(group) = elements.try_get_as_mut::<RadioGroupElement>(group.inner) {
            group.members.push(Radio { inner: cloned });
        }
        let selected = elements
            .try_get_as::<RadioGroupElement>(group.inner)
            .is_some_and(|group| group.selected == Some(Radio { inner: cloned }));
        let radio = elements.get_as_mut::<RadioElement>(cloned);
        radio.group = group;
        radio.set_accessibility_selection(selected);
    }

    // Previews clone their source children again, so create them only after
    // those children reference the completed subtree's groups.
    for (_, cloned) in pairs {
        elements.dispatch_mut(cloned, |element, elements| {
            if let Some(dropdown) = (element as &mut dyn Any).downcast_mut::<DropdownElement>() {
                dropdown.restore_selected_element(elements, gummy_tree, access_tree, by_internal_id);
            }
        });
    }
}
