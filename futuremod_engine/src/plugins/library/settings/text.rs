use mlua::{AnyUserData, Lua, UserData};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TextComponent {
    pub text: String,
}

pub fn create_text(_lua: &Lua, text: String) -> mlua::Result<TextComponent> {
    Ok(TextComponent::new(text))
}

impl TextComponent {
    pub fn new(text: String) -> TextComponent {
        TextComponent { text }
    }
}

impl UserData for TextComponent {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function(
            "withText",
            |_, (text_component, text): (AnyUserData, String)| {
                let original = text_component.borrow::<TextComponent>()?;
                let mut new_component = original.clone();
                new_component.text = text;
                Ok(new_component)
            },
        );
    }
}
