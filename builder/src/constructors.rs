#[cfg(feature = "audio")]
use std::path::Path;

#[cfg(feature = "audio")]
use retgui::elements::Audio;
#[cfg(feature = "code_highlighting")]
use retgui::elements::CodeEditor;
use retgui::elements::{Button, Calendar, Checkbox, CheckboxGroup, Container, Dropdown, Image, Radio, RadioGroup, Slider, Text, TextInput, TinyVg, Window};
use retgui::{App, ResourceId};

use crate::Builder;

pub fn button<S: 'static>(app: &mut App<S>) -> Builder<Button> {
    Builder::new(Button::new(app))
}

pub fn calendar<S: 'static>(app: &mut App<S>) -> Builder<Calendar> {
    Builder::new(Calendar::new(app))
}

pub fn checkbox<S: 'static>(app: &mut App<S>, label: &str, checked: bool) -> Builder<Checkbox> {
    Builder::new(Checkbox::new(app, label, checked))
}

pub fn checkbox_group<S: 'static>(app: &mut App<S>, label: &str) -> Builder<CheckboxGroup> {
    Builder::new(CheckboxGroup::new(app, label))
}

pub fn container<S: 'static>(app: &mut App<S>) -> Builder<Container> {
    Builder::new(Container::new(app))
}

pub fn dropdown<S: 'static>(app: &mut App<S>) -> Builder<Dropdown> {
    Builder::new(Dropdown::new(app))
}

pub fn image<S: 'static>(app: &mut App<S>, resource_id: ResourceId) -> Builder<Image> {
    Builder::new(Image::new(app, resource_id))
}

pub fn radio<S: 'static>(
    app: &mut App<S>,
    group: RadioGroup,
    value: &str,
    label: &str,
    selected: bool,
) -> Builder<Radio> {
    Builder::new(Radio::new(app, group, value, label, selected))
}

pub fn radio_group<S: 'static>(app: &mut App<S>, label: &str) -> Builder<RadioGroup> {
    Builder::new(RadioGroup::new(app, label))
}

pub fn slider<S: 'static>(app: &mut App<S>, thumb_size: f32) -> Builder<Slider> {
    Builder::new(Slider::new(app, thumb_size))
}

pub fn text<S: 'static>(app: &mut App<S>, text: &str) -> Builder<Text> {
    Builder::new(Text::new(app, text))
}

pub fn text_input<S: 'static>(app: &mut App<S>, text: &str) -> Builder<TextInput> {
    Builder::new(TextInput::new(app, text))
}

pub fn tiny_vg<S: 'static>(app: &mut App<S>, resource_id: ResourceId) -> Builder<TinyVg> {
    Builder::new(TinyVg::new(app, resource_id))
}

pub fn window<S: 'static>(app: &mut App<S>, title: &str) -> Builder<Window> {
    Builder::new(Window::new(app, title))
}

#[cfg(feature = "audio")]
pub fn audio<S: 'static>(app: &mut App<S>, path: &Path) -> Builder<Audio> {
    Builder::new(Audio::new(app, path))
}

#[cfg(feature = "code_highlighting")]
pub fn code_editor<S: 'static>(app: &mut App<S>, code: &str, extension: &str, theme: &str) -> Builder<CodeEditor> {
    Builder::new(CodeEditor::new(app, code, extension, theme))
}
