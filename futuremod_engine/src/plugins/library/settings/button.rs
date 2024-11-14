
use log::debug;
use mlua::{AnyUserData, Lua, UserData};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ButtonBuilder {
    pub(super) text: String,
    pub(super) disabled: bool,
    #[serde(skip)]
    pub(super) on_click: Option<mlua::Function>,
    #[serde(skip)]
    pub(super) manual_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Button {
    pub(super) text: String,
    pub(super) disabled: bool,
    #[serde(skip)]
    pub(super) on_click: Option<mlua::Function>,
    pub(super) id: String,
}

pub fn create_button(_: &Lua, text: String) -> mlua::Result<ButtonBuilder> {
    Ok(ButtonBuilder::new(text))
}

impl ButtonBuilder {
    pub fn new(text: String) -> ButtonBuilder {
        debug!("Create button with text '{}'", text);

        ButtonBuilder {
            text,
            disabled: false,
            on_click: None,
            manual_id: None,
        }
    }
}

impl UserData for ButtonBuilder {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("withText", |_, (builder, text): (AnyUserData, String)| {
            debug!("Change button text to '{}'", text);
            let builder = builder.borrow::<ButtonBuilder>()?;
            let mut new_buider = builder.clone();
            new_buider.text = text;
            Ok(new_buider)
        });

        methods.add_function(
            "withDisabled",
            |_, (builder, value): (AnyUserData, bool)| {
                debug!("Change button disabled to '{}'", value);
                let builder = builder.borrow::<ButtonBuilder>()?;
                let mut new_buider = builder.clone();
                new_buider.disabled = value;
                Ok(new_buider)
            },
        );

        methods.add_function("withID", |_, (builder, id): (AnyUserData, String)| {
            debug!("Change button ID to {}", id);
            let builder = builder.borrow::<ButtonBuilder>()?;
            let mut new_buider = builder.clone();
            new_buider.manual_id = Some(id);
            Ok(new_buider)
        });

        methods.add_function(
            "onClick",
            |_, (builder, cb): (AnyUserData, mlua::Function)| {
                debug!("Change button on click listener to '{:?}'", cb);
                let builder = builder.borrow::<ButtonBuilder>()?;
                let mut new_buider = builder.clone();
                new_buider.on_click = Some(cb);
                Ok(new_buider)
            },
        );
    }
}
