use mlua::UserData;
use serde::Serialize;

use super::{button::ButtonComponent, section::SectionComponent, text::TextComponent};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SettingsComponent {
    Button(ButtonComponent),
    Section(SectionComponent),
    Text(TextComponent),
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginSettings {
    pub components: Vec<SettingsComponent>,
}

impl UserData for PluginSettings {}