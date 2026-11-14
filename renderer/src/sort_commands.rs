use crate::{RenderCommand, RenderList};

#[derive(Debug)]
pub enum SortedItem {
    Overlay(SortedCommands),
    Other(u32),
}

#[derive(Debug)]
pub struct SortedCommands {
    pub children: Vec<SortedItem>,
}

impl SortedCommands {
    pub fn draw(render_list: &RenderList, overlay_render: &SortedCommands, on_draw: &mut dyn FnMut(&RenderCommand)) {
        let mut others = Vec::new();
        let mut overlays = Vec::new();

        for child in &overlay_render.children {
            match child {
                SortedItem::Other(_) => others.push(child),
                SortedItem::Overlay(_) => overlays.push(child),
            }
        }

        for child in others {
            if let SortedItem::Other(command_index) = child {
                let command = render_list.commands.get(*command_index as usize).unwrap();
                on_draw(command);
            }
        }

        for child in overlays {
            if let SortedItem::Overlay(overlay) = child {
                Self::draw(render_list, overlay, on_draw);
            }
        }
    }
}
