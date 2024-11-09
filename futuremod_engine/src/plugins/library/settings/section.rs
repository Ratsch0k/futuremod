use mlua::UserData;
use serde::Serialize;

use super::SettingsComponent;


#[derive(Debug, Clone, Serialize)]
pub struct SectionComponent {
    content: Vec<SettingsComponent>,
}

impl SectionComponent {
    pub fn new(content: Vec<SettingsComponent>) -> SectionComponent {
        SectionComponent { content }
    }
}

impl UserData for SectionComponent {}

