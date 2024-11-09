use std::{collections::{HashMap, HashSet}, sync::Arc};

use log::{debug, info};
use mlua::{AnyUserData, Lua, Table, UserData};
use serde::{Deserialize, Serialize};
use anyhow::anyhow;

use super::LuaResult;

#[derive(Debug, Clone)]
pub struct PluginSettingsLibrary {
    settings: Option<PluginSettings>,
}

impl PluginSettingsLibrary {
    pub fn new() -> Self {
        PluginSettingsLibrary{settings: None}
    }
}

impl UserData for PluginSettingsLibrary {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("create", |_lua, settings_library, components: Table| {
            let mut setting_components = Vec::<SettingsComponent>::new();

            for pair in components.pairs::<mlua::Value, mlua::Value>() {
                let (_key, value) = pair?;
                let component = value.as_userdata().ok_or(mlua::Error::RuntimeError("invalid component".into()))?;
        
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
        
            let settings = PluginSettings{components: setting_components};
        
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

fn create_settings(lua: &Lua, components: Table) -> LuaResult<()> {
    debug!("Create settings from: {:?}", components);

    let mut setting_components = Vec::<SettingsComponent>::new();

    for pair in components.pairs::<mlua::Value, mlua::Value>() {
        let (_key, value) = pair?;
        let component = value.as_userdata().ok_or(mlua::Error::RuntimeError("invalid component".into()))?;

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

    let settings = PluginSettings{components: setting_components};

    debug!("Created setting: {:#?}", settings);

    let globals = lua.globals();
    globals.set("_settings", settings)?;
    debug!("Stored settings in plugin globals");

    let other_globals = lua.globals();
    debug!("Stored settings: {:#?}", other_globals.get::<mlua::Value>("_settings"));

    lua.globals().for_each(|key: String, value: mlua::Value| {
        debug!("Setting Global: {key} -> {value:#?}");
        Ok(())
    })?;

    Ok(())
}

pub fn get_settings(context: &HashMap<&'static str, mlua::Value>) -> Result<Option<PluginSettings>, anyhow::Error> {
    let has_settings = context.contains_key("settings");

    if !has_settings {
        debug!("Plugin has no settings in globals");
        return Ok(None);
    }

    debug!("Settings key found in plugin's globals");

    let settings_global_value: &mlua::Value = context.get("settings")
        .ok_or(anyhow!("Could not access settings global in plugin context"))?;

    let settings_global = settings_global_value.as_userdata()
        .ok_or(anyhow!("Global settings has invalid format"))?;

    debug!("Got settings as userdata from globals");

    let settings = settings_global.borrow::<PluginSettingsLibrary>()
        .map_err(|e| anyhow!("Plugin context does not contain valid setting: {e}"))?;

    debug!("Could convert settings userdata to settings struct");

    Ok(settings.settings.clone())
}

#[derive(Debug, Clone, Serialize)]
pub struct ButtonComponent {
    text: String,
    disabled: bool,
    #[serde(skip)]
    on_click: Option<mlua::Function>,
    id: Option<String>,
}

fn create_button(_: &Lua, text: String) -> LuaResult<ButtonComponent> {
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

        methods.add_function("withDisabled", |_, (builder, value): (AnyUserData, bool)| {
            debug!("Change button disabled to '{}'", value);
            let builder = builder.borrow::<ButtonComponent>()?;
            let mut new_buider = builder.clone();
            new_buider.disabled = value;
            Ok(new_buider)
        });

        methods.add_function("withID", |_, (builder, id): (AnyUserData, String)| {
            debug!("Change button ID to {}", id);
            let builder = builder.borrow::<ButtonComponent>()?;
            let mut new_buider = builder.clone();
            new_buider.id = Some(id);
            Ok(new_buider)
        });

        methods.add_function("onClick", |_, (builder, cb): (AnyUserData, mlua::Function)| {
            debug!("Change button on click listener to '{:?}'", cb);
            let builder = builder.borrow::<ButtonComponent>()?;
            let mut new_buider = builder.clone();
            new_buider.on_click = Some(cb);
            Ok(new_buider)
        });
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SectionComponent {
    content: Vec<SettingsComponent>
}

impl SectionComponent {
    pub fn new(content: Vec<SettingsComponent>) -> SectionComponent {
        SectionComponent{content}
    }
}

impl UserData for SectionComponent {}


#[derive(Debug, Clone, Serialize)]
pub struct TextComponent {
    pub text: String
}

fn create_text(_lua: &Lua, text: String) -> LuaResult<TextComponent> {
    Ok(TextComponent::new(text))
}

impl TextComponent {
    pub fn new(text: String) -> TextComponent {
        TextComponent{text}
    }
}

impl UserData for TextComponent {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("withText", |_, (text_component, text): (AnyUserData, String)| {
            let original = text_component.borrow::<TextComponent>()?;
            let mut new_component = original.clone();
            new_component.text = text;
            Ok(new_component)
        });
    }
}