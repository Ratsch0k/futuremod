use std::collections::HashMap;

use futuremod_data::plugin::{Plugin, PluginState};
use iced::{widget::{column, container, row, text, Toggler}, Alignment, Length, Padding};
use iced_aw::Badge;
use iced_fonts::Bootstrap;
use log::warn;

use crate::{theme, widget::{bold, button, icon_text_button, icon_text_button_advanced, Column, Element, IconTextButtonOptions, Row}};

use super::view::Message;

pub fn plugin_overview<'a>(plugins: &'a HashMap<String, Plugin>, is_developer: bool) -> Element<'a, Message> {
  column![
    plugin_overview_heading(is_developer),
    plugin_list(plugins),
  ]
    .spacing(16)
    .padding(16)
    .into()
}

fn plugin_overview_heading<'a>(is_developer: bool) -> Element<'a, Message> {
  row![
    container(text("Plugins").size(24)).width(Length::Fill),
    plugin_overview_actions(is_developer),
  ]
    .width(Length::Fill)
    .into()
}

fn plugin_overview_actions<'a>(is_developer: bool) -> Element<'a, Message> {
  let mut actions = Row::new();

  if is_developer {
    actions = actions.push(
      icon_text_button(Bootstrap::Bug, "Install as Developer")
        .class(theme::Button::Default)
        .on_press(Message::DevInstall)
    );
  }

  actions = actions.push(
    icon_text_button_advanced(Bootstrap::Plus, "Install", IconTextButtonOptions::new().with_icon_size(24))
      .class(theme::Button::Primary)
      .padding(Padding{top: 3.0, right: 16.0, bottom: 3.0, left: 12.0})
      .on_press(Message::Install)
  );

  actions
    .spacing(8.0)
    .into()
}

fn plugin_list<'a>(plugins: &'a HashMap<String, Plugin>) -> Element<'a, Message> {
  let mut list = Column::new();

  let mut keys: Vec<&String> = plugins.keys().collect();
  keys.sort();

  for name in keys {
    match plugins.get(name) {
      Some(plugin) => {
        list = list.push(plugin_card(name, plugin));
      },
      None => {
        warn!("Missing plugin while generating plugin list");
      }
    }
  }

  list
    .spacing(16.0)
    .into()
}

fn plugin_card<'a>(name: &'a String, plugin: &Plugin) -> Element<'a, Message> {
  container(
    row![
      Column::new()
        .push(text(name).size(20))
        .push(plugin_card_description(&plugin))
        .width(Length::Fill)
        .spacing(8.0),
      plugin_card_actions(&plugin),
    ]
    .align_y(Alignment::Center)
  )
  .class(theme::Container::Box)
  .padding(16)
  .into()
}

fn plugin_card_actions<'a>(plugin: &Plugin) -> Element<'a, Message> {
  Row::new()
    .push(plugin_go_to_details_button(&plugin))
    .push_maybe(plugin_toggle_button(&plugin))
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn plugin_card_description<'a>(plugin: &Plugin) -> Element<'a, Message> {
  let dev_mode_badge: Option<Element<'a, Message>> = match &plugin.in_dev_mode {
    false => None,
    true => Some(Badge::new(text("Developer Mode").font(bold()).size(12))
      .style(|theme: &theme::Theme, status| iced_aw::style::badge::warning(&theme.theme, status))
      .padding(4)
      .into()),
  };

  Row::new()
    .push(plugin_state_component(plugin))
    .push_maybe(dev_mode_badge)
    .align_y(Alignment::Center)
    .spacing(4.0)
    .into()
}

fn plugin_state_component<'a>(plugin: &Plugin) -> Element<'a, Message> {
  let message = match &plugin.state {
    PluginState::Error(_) => String::from("Error"),
    PluginState::Unloaded => String::from("Unloaded"),
    _ => {
      if plugin.enabled {
        String::from("Enabled")
      } else {
        String::from("Disabled")
      }
    }
  };

  text(message)
    .size(14)
    .into()
}

fn plugin_go_to_details_button<'a>(plugin: &Plugin) -> Element<'a, Message> {
  button(text("Details"))
    .on_press(Message::ToPlugin(plugin.info.name.clone()))
    .class(theme::Button::Default)
    .into()
}

fn plugin_toggle_button<'a>(plugin: &Plugin) -> Option<Element<'a, Message>> {
  if let PluginState::Error(_) = plugin.state {
    return None;
  }

  let label = match plugin.enabled {
    true => "Enabled",
    false => "Disabled",
  };

  let plugin_name = plugin.info.name.clone();
  let enabled = plugin.enabled;

  Some(
    container(
      Toggler::new(enabled)
        .label(label)
        .on_toggle(move |state| match state {
          true => Message::Enable(plugin_name.clone()),
          false => Message::Disable(plugin_name.clone()),
        })
        .width(120)
  ).into()
  )
}
