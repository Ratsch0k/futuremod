use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use anyhow::{anyhow, bail};
use log::debug;
use mlua::UserData;
use serde::{ser::SerializeStruct, Serialize};
use uuid::Uuid;

use super::{
    button::{Button, ButtonBuilder},
    section::{Section, SectionBuilder},
    text::{Text, TextBuilder},
};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ComponentBuilder {
    Button(ButtonBuilder),
    Section(SectionBuilder),
    Text(TextBuilder),
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Component {
    Button(Button),
    Section(Section),
    Text(Text),
}

#[derive(Debug, Clone)]
pub struct PluginSettings {
    components: HashMap<String, Arc<Component>>,
    root: Weak<Component>,
}

impl Serialize for PluginSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("settings", 1)?;
        let strong_root = self
            .root
            .upgrade()
            .ok_or(serde::ser::Error::custom("Root dropped"))?;
        state.serialize_field("root", &*strong_root)?;
        state.end()
    }
}

impl UserData for PluginSettings {}

pub(super) fn generate_new_id<T>(map: &HashMap<String, T>) -> String {
    let mut key = Uuid::new_v4().to_string();

    while map.contains_key(&key) {
        key = Uuid::new_v4().to_string();
    }

    key
}

pub(super) fn create_settings(components: Vec<mlua::Value>) -> mlua::Result<PluginSettings> {
    debug!("Create settings from: {:?}", components);

    let mut setting_components = HashMap::<String, Arc<Component>>::new();

    let mut root_components = Vec::<Weak<Component>>::new();

    for unknown_component in components.into_iter() {
        let component = build_component(&mut setting_components, unknown_component)
            .map_err(|e| mlua::Error::RuntimeError(format!("{:?}", e)))?;

        root_components.push(component);
    }

    let id = generate_new_id(&setting_components);
    let root_section = Section {
        content: root_components,
        id: id.clone(),
    };

    let shared_root = Arc::new(Component::Section(root_section));

    setting_components.insert(id, shared_root.clone());

    let settings = PluginSettings {
        components: setting_components,
        root: Arc::downgrade(&shared_root),
    };

    Ok(settings)
}

fn build_component(
    components: &mut HashMap<String, Arc<Component>>,
    component: mlua::Value,
) -> Result<Weak<Component>, anyhow::Error> {
    let userdata = component
        .as_userdata()
        .ok_or(anyhow!("Component not a userdata"))?;

    if let Ok(button_builder) = userdata.borrow::<ButtonBuilder>() {
        build_button(components, button_builder.clone())
    } else if let Ok(section_builder) = userdata.borrow::<SectionBuilder>() {
        build_section(components, section_builder.clone())
    } else {
        bail!("Unkown component: {:?}", userdata);
    }
}

fn build_button(
    components: &mut HashMap<String, Arc<Component>>,
    button_builder: ButtonBuilder,
) -> Result<Weak<Component>, anyhow::Error> {
    let id = match button_builder.manual_id {
        Some(v) => {
            if components.contains_key(&v) {
                bail!("Duplicate id");
            }

            v
        }
        None => generate_new_id(&components),
    };

    let button = Button {
        text: button_builder.text,
        disabled: button_builder.disabled,
        on_click: button_builder.on_click,
        id: id.clone(),
    };

    let shared_component = Arc::new(Component::Button(button));

    components.insert(id, shared_component.clone());

    Ok(Arc::downgrade(&shared_component))
}

fn build_known_compoment(
    components: &mut HashMap<String, Arc<Component>>,
    builder: ComponentBuilder,
) -> Result<Weak<Component>, anyhow::Error> {
    match builder {
        ComponentBuilder::Button(button_builder) => build_button(components, button_builder),
        ComponentBuilder::Section(section_builder) => build_section(components, section_builder),
        ComponentBuilder::Text(text_builder) => build_text(components, text_builder),
    }
}

fn build_section(
    components: &mut HashMap<String, Arc<Component>>,
    section_builder: SectionBuilder,
) -> Result<Weak<Component>, anyhow::Error> {
    let mut content = Vec::<Weak<Component>>::new();

    for component_builder in section_builder.content {
        let component = build_known_compoment(components, component_builder)?;

        content.push(component);
    }

    let id = generate_new_id(&components);
    let section = Section {
        id: id.clone(),
        content,
    };

    let shared_component = Arc::new(Component::Section(section));

    components.insert(id, shared_component.clone());

    Ok(Arc::downgrade(&shared_component))
}

fn build_text(
    components: &mut HashMap<String, Arc<Component>>,
    text_builder: TextBuilder,
) -> Result<Weak<Component>, anyhow::Error> {
    let id = generate_new_id(components);
    let text = Arc::new(Component::Text(Text {
        id: id.clone(),
        text: text_builder.text,
    }));

    components.insert(id, text.clone());

    Ok(Arc::downgrade(&text))
}
