#![cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use retgui::drivers::headless::run;
use retgui::elements::{Element, Text, Window};
use retgui::geometry::{Point, Size};
use retgui::{RendererType, px};

use retgui_builder::{ElementBuilder, TextBuilder};

use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, KeyCode, KeyLocation, ModifiersState, PhysicalKey};

#[test]
fn copy_mouse_selected_text_to_clipboard() {
    run("text_copy_test", |test| {
        test.set_clipboard_text("clipboard sentinel");
        let text = Text::new("Copy this text")
            .font_size(32.0)
            .width(px(300))
            .selectable(true);
        let window = Window::new_with_renderer("Text copy", RendererType::VelloCPU).push(text.clone());
        test.open(&window, Size::new(800.0, 600.0));

        let bounds = text.get_computed_box_transformed().content_rectangle();
        let y = (bounds.y + bounds.height / 2.0) as f64;
        test.pointer_move(&window, Point::new(bounds.x as f64, y));
        test.pointer_down(&window);
        test.pointer_move(&window, Point::new((bounds.x + bounds.width - 1.0) as f64, y));
        test.pointer_up(&window);
        test.modifiers_changed(
            &window,
            if cfg!(target_os = "macos") {
                ModifiersState::META
            } else {
                ModifiersState::CONTROL
            },
        );
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
        test.modifiers_changed(&window, ModifiersState::empty());

        assert_eq!(Text::text(&text), "Copy this text", "copy must not modify the source");
        assert_eq!(
            test.clipboard_text(),
            "Copy this text",
            "copy must put the mouse selection on the clipboard"
        );
    });
}

#[test]
fn paste_and_cut_text_input() {
    use retgui::elements::TextInput;
    run("clipboard_input_test", |test| {
        let input = TextInput::new("").width(px(300));
        let window = Window::new_with_renderer("Clipboard input", RendererType::VelloCPU).push(input.clone());
        test.open(&window, Size::new(800.0, 600.0));
        test.click(&input);
        let content = "Unicode: \u{65e5}\u{672c}\u{8a9e}";
        test.set_clipboard_text(content);
        test.modifiers_changed(
            &window,
            if cfg!(target_os = "macos") {
                ModifiersState::META
            } else {
                ModifiersState::CONTROL
            },
        );
        let mut shortcut = |key: &str, code| {
            for state in [ElementState::Pressed, ElementState::Released] {
                test.keyboard_input(
                    &window,
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        logical_key: Key::Character(key.into()),
                        text: None,
                        location: KeyLocation::Standard,
                        state,
                        repeat: false,
                        text_with_all_modifiers: None,
                        key_without_modifiers: Key::Character(key.into()),
                    },
                );
            }
        };
        shortcut("v", KeyCode::KeyV);
        assert_eq!(input.get_text(), content);
        shortcut("a", KeyCode::KeyA);
        shortcut("x", KeyCode::KeyX);
        assert_eq!(input.get_text(), "");
        assert_eq!(test.clipboard_text(), content);
    });
}

fn send_key(
    test: &mut retgui::drivers::headless::HeadlessApp,
    window: &Window,
    key: &str,
    code: KeyCode,
    modifiers: ModifiersState,
) {
    test.modifiers_changed(window, modifiers);
    for state in [ElementState::Pressed, ElementState::Released] {
        test.keyboard_input(
            window,
            KeyEvent {
                physical_key: PhysicalKey::Code(code),
                logical_key: Key::Character(key.into()),
                text: None,
                location: KeyLocation::Standard,
                state,
                repeat: false,
                text_with_all_modifiers: None,
                key_without_modifiers: Key::Character(key.into()),
            },
        );
    }
    test.modifiers_changed(window, ModifiersState::empty());
}
fn action() -> ModifiersState {
    if cfg!(target_os = "macos") {
        ModifiersState::META
    } else {
        ModifiersState::CONTROL
    }
}
fn history_case(
    name: &str,
    check: impl FnOnce(&mut retgui::drivers::headless::HeadlessApp, &Window, &retgui::elements::TextInput),
) {
    run(name, |test| {
        let input = retgui::elements::TextInput::new("").width(px(300));
        let window = Window::new_with_renderer("Undo history", RendererType::VelloCPU).push(input.clone());
        test.open(&window, Size::new(800.0, 600.0));
        test.click(&input);
        check(test, &window, &input);
    });
}
#[test]
fn typing_undo_redo_and_new_edit_clears_redo() {
    history_case("typing_history", |test, window, input| {
        for (key, code) in [("a", KeyCode::KeyA), ("b", KeyCode::KeyB)] {
            send_key(test, window, key, code, ModifiersState::empty());
        }
        assert_eq!(input.get_text(), "ab");
        send_key(test, window, "z", KeyCode::KeyZ, action());
        assert_eq!(input.get_text(), "");
        send_key(test, window, "z", KeyCode::KeyZ, action() | ModifiersState::SHIFT);
        assert_eq!(input.get_text(), "ab");
        send_key(test, window, "z", KeyCode::KeyZ, action());
        send_key(test, window, "c", KeyCode::KeyC, ModifiersState::empty());
        send_key(test, window, "z", KeyCode::KeyZ, action() | ModifiersState::SHIFT);
        assert_eq!(input.get_text(), "c");
    });
}
#[test]
fn unicode_paste_undo_redo() {
    history_case("paste_history", |test, window, input| {
        let content = "\u{65e5}\u{672c}\u{8a9e} \u{1f600}\nsecond line";
        test.set_clipboard_text(content);
        send_key(test, window, "v", KeyCode::KeyV, action());
        assert_eq!(input.get_text(), content);
        for _ in 0..2 {
            send_key(test, window, "z", KeyCode::KeyZ, action());
            assert_eq!(input.get_text(), "");
            send_key(test, window, "z", KeyCode::KeyZ, action() | ModifiersState::SHIFT);
            assert_eq!(input.get_text(), content);
        }
    });
}
#[test]
fn paste_over_selection_restores_text_and_selection_on_undo() {
    history_case("replace_history", |test, window, input| {
        test.type_text(window, "original");
        send_key(test, window, "a", KeyCode::KeyA, action());
        test.set_clipboard_text("replacement");
        send_key(test, window, "v", KeyCode::KeyV, action());
        assert_eq!(input.get_text(), "replacement");
        send_key(test, window, "z", KeyCode::KeyZ, action());
        assert_eq!(input.get_text(), "original");
        send_key(test, window, "z", KeyCode::KeyZ, action() | ModifiersState::SHIFT);
        assert_eq!(input.get_text(), "replacement");
        send_key(test, window, "z", KeyCode::KeyZ, action());
        send_key(test, window, "x", KeyCode::KeyX, ModifiersState::empty());
        assert_eq!(input.get_text(), "x", "undo must restore the replaced selection");
    });
}
#[test]
fn typing_and_paste_preserve_existing_insertion_grouping() {
    history_case("mixed_history", |test, window, input| {
        send_key(test, window, "a", KeyCode::KeyA, ModifiersState::empty());
        test.set_clipboard_text("paste");
        send_key(test, window, "v", KeyCode::KeyV, action());
        send_key(test, window, "b", KeyCode::KeyB, ModifiersState::empty());
        assert_eq!(input.get_text(), "apasteb");
        // Existing TextCommand::merge groups adjacent insertions, including paste.
        send_key(test, window, "z", KeyCode::KeyZ, action());
        assert_eq!(input.get_text(), "");
        send_key(test, window, "z", KeyCode::KeyZ, action() | ModifiersState::SHIFT);
        assert_eq!(input.get_text(), "apasteb");
    });
}
