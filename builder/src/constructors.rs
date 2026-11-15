#[cfg(feature = "audio")]
use std::path::Path;

use retgui::ResourceId;
#[cfg(feature = "audio")]
use retgui::elements::Audio;
#[cfg(feature = "code_highlighting")]
use retgui::elements::CodeEditor;
use retgui::elements::{Button, Calendar, Checkbox, CheckboxGroup, Container, Dropdown, Image, Radio, RadioGroup, Slider, Text, TextInput, TinyVg, Window};

use crate::Builder;

pub fn button() -> Builder<Button> {
    Builder::new(Button::new())
}

pub fn calendar() -> Builder<Calendar> {
    Builder::new(Calendar::new())
}

pub fn checkbox(label: &str, checked: bool) -> Builder<Checkbox> {
    Builder::new(Checkbox::new(label, checked))
}

pub fn checkbox_group(label: &str) -> Builder<CheckboxGroup> {
    Builder::new(CheckboxGroup::new(label))
}

pub fn container() -> Builder<Container> {
    Builder::new(Container::new())
}

pub fn dropdown() -> Builder<Dropdown> {
    Builder::new(Dropdown::new())
}

pub fn image(resource_id: ResourceId) -> Builder<Image> {
    Builder::new(Image::new(resource_id))
}

pub fn radio(active_value: std::rc::Rc<std::cell::RefCell<String>>, value: &str, label: &str) -> Builder<Radio> {
    Builder::new(Radio::new(value, label, active_value))
}

pub fn radio_group(label: &str) -> Builder<RadioGroup> {
    Builder::new(RadioGroup::new(label))
}

pub fn slider(thumb_size: f32) -> Builder<Slider> {
    Builder::new(Slider::new(thumb_size))
}

pub fn text(text: &str) -> Builder<Text> {
    Builder::new(Text::new(text))
}

pub fn text_input(text: &str) -> Builder<TextInput> {
    Builder::new(TextInput::new(text))
}

pub fn tiny_vg(resource_id: ResourceId) -> Builder<TinyVg> {
    Builder::new(TinyVg::new(resource_id))
}

pub fn window(title: &str) -> Builder<Window> {
    Builder::new(Window::new(title))
}

#[cfg(feature = "audio")]
pub fn audio(path: &Path) -> Builder<Audio> {
    Builder::new(Audio::new(path))
}

#[cfg(feature = "code_highlighting")]
pub fn code_editor(code: &str, extension: &str, theme: &str) -> Builder<CodeEditor> {
    Builder::new(CodeEditor::new(code, extension, theme))
}
