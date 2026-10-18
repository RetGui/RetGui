#![cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]

use clipboard_rs::{Clipboard, ClipboardContext};

use retgui::drivers::headless::run;
use retgui::elements::{Element, Text, Window};
use retgui::geometry::{Point, Size};
use retgui::{RendererType, px};

use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{Key, KeyCode, KeyLocation, ModifiersState, PhysicalKey};

#[test]
#[ignore = "requires a desktop clipboard"]
fn copy_mouse_selected_text_to_clipboard() {
    let clipboard = ClipboardContext::new().expect("desktop clipboard must be available");
    clipboard.set_text("clipboard sentinel".into()).unwrap();

    run(
        "text_copy_test",
        (),
        |app, _| {
            let text = Text::new(app, "Copy this text");
            text.set_font_size(app, 32.0);
            text.set_width(app, px(300));
            text.set_selectable(app, true);
            let window = Window::new_with_renderer(app, "Text copy", RendererType::VelloCPU);
            window.push(app, text);
            (text, window)
        },
        |test, (text, window)| {
            test.open(&window, Size::new(800.0, 600.0));
            let bounds = text.computed_box_transformed(test.app_mut()).content_rectangle();
            let y = (bounds.y + bounds.height / 2.0) as f64;
            test.pointer_move(&window, Point::new(bounds.x as f64, y));
            test.pointer_down(&window);
            test.pointer_move(&window, Point::new((bounds.x + bounds.width - 1.0) as f64, y));
            test.pointer_up(&window);

            let modifiers = if cfg!(target_os = "macos") {
                ModifiersState::META
            } else {
                ModifiersState::CONTROL
            };
            let (app, state) = test.parts_mut();
            let _ = app.on_window_event(window, WindowEvent::ModifiersChanged(modifiers.into()), state);
            for state in [ElementState::Pressed, ElementState::Released] {
                test.keyboard_input(
                    &window,
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::KeyC),
                        logical_key: Key::Character("c".into()),
                        text: None,
                        location: KeyLocation::Standard,
                        state,
                        repeat: false,
                        text_with_all_modifiers: None,
                        key_without_modifiers: Key::Character("c".into()),
                    },
                );
            }

            assert_eq!(
                text.text(test.app()),
                "Copy this text",
                "copy must not modify the source"
            );
            assert_eq!(
                clipboard.get_text().unwrap(),
                "Copy this text",
                "the copy shortcut must put the mouse selection on the clipboard"
            );
        },
    );
}
