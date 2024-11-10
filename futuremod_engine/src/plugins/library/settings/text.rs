use mlua::{AnyUserData, Lua, UserData};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TextBuilder {
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Text {
    pub(super) id: String,
    pub(super) text: String,
}

pub fn create_text(_lua: &Lua, text: String) -> mlua::Result<TextBuilder> {
    Ok(TextBuilder::new(text))
}

impl TextBuilder {
    pub fn new(text: String) -> TextBuilder {
        TextBuilder { text }
    }
}

impl UserData for TextBuilder {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function(
            "withText",
            |_, (text_component, text): (AnyUserData, String)| {
                let original = text_component.borrow::<TextBuilder>()?;
                let mut new_component = original.clone();
                new_component.text = text;
                Ok(new_component)
            },
        );
    }
}
