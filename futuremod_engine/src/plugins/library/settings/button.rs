use log::debug;
use mlua::{AnyUserData, Lua, UserData};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ButtonComponent {
    text: String,
    disabled: bool,
    #[serde(skip)]
    on_click: Option<mlua::Function>,
    id: Option<String>,
}

pub fn create_button(_: &Lua, text: String) -> mlua::Result<ButtonComponent> {
    Ok(ButtonComponent::new(text))
}

impl ButtonComponent {
    pub fn new(text: String) -> ButtonComponent {
        debug!("Create button with text '{}'", text);

        ButtonComponent {
            text,
            disabled: false,
            on_click: None,
            id: None,
        }
    }
}

impl UserData for ButtonComponent {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("withText", |_, (builder, text): (AnyUserData, String)| {
            debug!("Change button text to '{}'", text);
            let builder = builder.borrow::<ButtonComponent>()?;
            let mut new_buider = builder.clone();
            new_buider.text = text;
            Ok(new_buider)
        });

        methods.add_function(
            "withDisabled",
            |_, (builder, value): (AnyUserData, bool)| {
                debug!("Change button disabled to '{}'", value);
                let builder = builder.borrow::<ButtonComponent>()?;
                let mut new_buider = builder.clone();
                new_buider.disabled = value;
                Ok(new_buider)
            },
        );

        methods.add_function("withID", |_, (builder, id): (AnyUserData, String)| {
            debug!("Change button ID to {}", id);
            let builder = builder.borrow::<ButtonComponent>()?;
            let mut new_buider = builder.clone();
            new_buider.id = Some(id);
            Ok(new_buider)
        });

        methods.add_function(
            "onClick",
            |_, (builder, cb): (AnyUserData, mlua::Function)| {
                debug!("Change button on click listener to '{:?}'", cb);
                let builder = builder.borrow::<ButtonComponent>()?;
                let mut new_buider = builder.clone();
                new_buider.on_click = Some(cb);
                Ok(new_buider)
            },
        );
    }
}