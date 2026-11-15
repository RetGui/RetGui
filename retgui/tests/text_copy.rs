#![cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]

use clipboard_rs::{Clipboard, ClipboardContext};
use retgui::drivers::headless::run;
use retgui::elements::{Element, Text, Window};
use retgui::geometry::{Point, Size};
use retgui::{RendererType, px};
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, KeyCode, KeyLocation, ModifiersState, PhysicalKey};

#[test]
#[ignore = "requires and modifies a desktop clipboard"]
fn copy_mouse_selected_text_to_clipboard() {
    let clipboard = ClipboardContext::new().expect("desktop clipboard must be available");
    clipboard.set_text("clipboard sentinel".into()).unwrap();

    run("text_copy_test", |test| {
        let text = Text::new("Copy this text").font_size(32.0).width(px(300)).selectable(true);
        let window = Window::new_with_renderer("Text copy", RendererType::VelloCPU).push(text.clone());
        test.open(&window, Size::new(800.0, 600.0));

        let bounds = text.get_computed_box_transformed().content_rectangle();
        let y = (bounds.y + bounds.height / 2.0) as f64;
        test.pointer_move(&window, Point::new(bounds.x as f64, y));
        test.pointer_down(&window);
        test.pointer_move(&window, Point::new((bounds.x + bounds.width - 1.0) as f64, y));
        test.pointer_up(&window);
        test.modifiers_changed(&window, if cfg!(target_os = "macos") {
            ModifiersState::META
        } else {
            ModifiersState::CONTROL
        });
        for state in [ElementState::Pressed, ElementState::Released] {
            test.keyboard_input(&window, KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::KeyC),
                logical_key: Key::Character("c".into()),
                text: None,
                location: KeyLocation::Standard,
                state,
                repeat: false,
                text_with_all_modifiers: None,
                key_without_modifiers: Key::Character("c".into()),
            });
        }
        test.modifiers_changed(&window, ModifiersState::empty());

        assert_eq!(text.get_text(), "Copy this text", "copy must not modify the source");
        assert_eq!(clipboard.get_text().unwrap(), "Copy this text", "copy must put the mouse selection on the clipboard");
    });
}
