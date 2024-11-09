use std::{
    collections::HashMap,
    sync::Arc,
};

use anyhow::anyhow;
use log::debug;
use mlua::{Lua, Table, UserData};

mod settings;
mod button;
mod section;
mod text;

pub use settings::PluginSettings;

use settings::SettingsComponent;
use button::{create_button, ButtonComponent};
use text::{create_text, TextComponent};
use section::SectionComponent;

use super::LuaResult;

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
        methods.add_method_mut("create", |_lua, settings_library, components: Table| {
            let mut setting_components = Vec::<SettingsComponent>::new();

            for pair in components.pairs::<mlua::Value, mlua::Value>() {
                let (_key, value) = pair?;
                let component = value
                    .as_userdata()
                    .ok_or(mlua::Error::RuntimeError("invalid component".into()))?;

                if component.is::<ButtonComponent>() {
                    debug!("Got button builder component");

                    let builder = component.borrow::<ButtonComponent>()?;
                    setting_components.push(SettingsComponent::Button(builder.clone()));
                } else if component.is::<TextComponent>() {
                    debug!("Got text compoment");

                    let builder = component.borrow::<TextComponent>()?;
                    setting_components.push(SettingsComponent::Text(builder.clone()));
                } else {
                    debug!("Got unknown compomnent: {:?}", value);
                }
            }

            let settings = PluginSettings {
                components: setting_components,
            };

            settings_library.settings = Some(settings);

            Ok(())
        });

        methods.add_function("Button", create_button);
        methods.add_function("Text", create_text);
    }
}

pub fn create_settings_library_new(_lua: Arc<Lua>) -> Result<PluginSettingsLibrary, mlua::Error> {
    Ok(PluginSettingsLibrary::new())
}

pub fn create_settings_library(lua: Arc<Lua>) -> Result<Table, mlua::Error> {
    let library = lua.create_table()?;
    library.set("create", lua.create_function(create_settings)?)?;
    library.set("Button", lua.create_function(create_button)?)?;
    library.set("Text", lua.create_function(create_text)?)?;

    Ok(library)
}

pub fn create_settings_table(lua: &Lua) -> Result<mlua::Value, mlua::Error> {
    let library = lua.create_table()?;
    library.set("create", lua.create_function(create_settings)?)?;
    library.set("Button", lua.create_function(create_button)?)?;
    library.set("Text", lua.create_function(create_text)?)?;

    Ok(mlua::Value::Table(library))
}


fn create_settings(lua: &Lua, components: Table) -> LuaResult<()> {
    debug!("Create settings from: {:?}", components);

    let mut setting_components = Vec::<SettingsComponent>::new();

    for pair in components.pairs::<mlua::Value, mlua::Value>() {
        let (_key, value) = pair?;
        let component = value
            .as_userdata()
            .ok_or(mlua::Error::RuntimeError("invalid component".into()))?;

        if component.is::<ButtonComponent>() {
            debug!("Got button builder component");

            let builder = component.borrow::<ButtonComponent>()?;
            setting_components.push(SettingsComponent::Button(builder.clone()));
        } else if component.is::<TextComponent>() {
            debug!("Got text compoment");

            let builder = component.borrow::<TextComponent>()?;
            setting_components.push(SettingsComponent::Text(builder.clone()));
        } else if component.is::<SectionComponent>() {
            debug!("Got section compoment");

            let builder = component.borrow::<SectionComponent>()?;
            setting_components.push(SettingsComponent::Section(builder.clone()));
        } else {
            debug!("Got unknown compomnent: {:?}", value);
        }
    }

    let settings = PluginSettings {
        components: setting_components,
    };

    debug!("Created setting: {:#?}", settings);

    let globals = lua.globals();
    globals.set("_settings", settings)?;
    debug!("Stored settings in plugin globals");

    let other_globals = lua.globals();
    debug!(
        "Stored settings: {:#?}",
        other_globals.get::<mlua::Value>("_settings")
    );

    lua.globals().for_each(|key: String, value: mlua::Value| {
        debug!("Setting Global: {key} -> {value:#?}");
        Ok(())
    })?;

    Ok(())
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


