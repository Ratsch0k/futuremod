use std::sync::Weak;

use mlua::{Lua, UserData};
use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Serialize,
};

use super::{button::ButtonBuilder, settings::Component, text::TextBuilder, ComponentBuilder};

#[derive(Debug, Clone, Serialize)]
pub struct SectionBuilder {
    pub(super) content: Vec<ComponentBuilder>,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub(super) content: Vec<Weak<Component>>,
    pub(super) id: String,
}

pub struct SectionContent<'a>(&'a Vec<Weak<Component>>);

impl<'a> Serialize for SectionContent<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut content_state = serializer.serialize_seq(Some(self.0.len()))?;

        for child in self.0.iter() {
            let strong_child = child
                .upgrade()
                .ok_or(serde::ser::Error::custom("Child dropped"))?;

            content_state.serialize_element(&*strong_child)?;
        }

        content_state.end()
    }
}

impl Serialize for Section {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("section", 2)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("content", &SectionContent(&self.content))?;

        state.end()
    }
}

impl SectionBuilder {
    pub fn new(content: Vec<ComponentBuilder>) -> SectionBuilder {
        SectionBuilder { content }
    }
}

pub fn create_section(_lua: &Lua, children: Vec<mlua::Value>) -> mlua::Result<SectionBuilder> {
    let mut components = Vec::<ComponentBuilder>::new();

    for child in children {
        let child_userdata = child
            .as_userdata()
            .ok_or(mlua::Error::RuntimeError("Component not userdata".into()))?;

        let compoment = if let Ok(v) = child_userdata.borrow::<ButtonBuilder>() {
            ComponentBuilder::Button(v.clone())
        } else if let Ok(v) = child_userdata.borrow::<TextBuilder>() {
            ComponentBuilder::Text(v.clone())
        } else if let Ok(v) = child_userdata.borrow::<SectionBuilder>() {
            ComponentBuilder::Section(v.clone())
        } else {
            return Err(mlua::Error::RuntimeError("Unknown compoment".into()));
        };

        components.push(compoment)
    }

    Ok(SectionBuilder {
        content: components,
    })
}

impl UserData for SectionBuilder {}
