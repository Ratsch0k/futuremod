use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use log::debug;
use mlua::{Lua, UserData};

mod button;
mod section;
mod settings;
mod text;

pub use settings::PluginSettings;

use button::create_button;
use section::create_section;
use settings::{create_settings, ComponentBuilder};
use text::create_text;

#[derive(Debug, Clone)]
pub struct PluginSettingsLibrary {
    settings: Option<PluginSettings>,
}

impl PluginSettingsLibrary {
    pub fn new() -> Self {
        PluginSettingsLibrary { settings: None }
    }
}

impl UserData for PluginSettingsLibrary {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "create",
            |_lua, settings_library, components: Vec<mlua::Value>| {
                let settings = create_settings(components)?;

                settings_library.settings = Some(settings);

                Ok(())
            },
        );

        methods.add_function("Button", create_button);
        methods.add_function("Text", create_text);
        methods.add_function("Section", create_section);
    }
}

pub fn create_settings_library_new(_lua: Arc<Lua>) -> Result<PluginSettingsLibrary, mlua::Error> {
    Ok(PluginSettingsLibrary::new())
}

pub fn get_settings(
    context: &HashMap<&'static str, mlua::Value>,
) -> Result<Option<PluginSettings>, anyhow::Error> {
    let has_settings = context.contains_key("settings");

    if !has_settings {
        debug!("Plugin has no settings in globals");
        return Ok(None);
    }

    debug!("Settings key found in plugin's globals");

    let settings_global_value: &mlua::Value = context.get("settings").ok_or(anyhow!(
        "Could not access settings global in plugin context"
    ))?;

    let settings_global = settings_global_value
        .as_userdata()
        .ok_or(anyhow!("Global settings has invalid format"))?;

    debug!("Got settings as userdata from globals");

    let settings = settings_global
        .borrow::<PluginSettingsLibrary>()
        .map_err(|e| anyhow!("Plugin context does not contain valid setting: {e}"))?;

    debug!("Could convert settings userdata to settings struct");

    Ok(settings.settings.clone())
}
