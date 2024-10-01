use futuremod_data::plugin::{Plugin, PluginDependency, PluginState};
use iced::{widget::{column, container, row, rule, text, Scrollable, Toggler}, Alignment, Length};
use iced_aw::BootstrapIcon;

use crate::{theme::{self, Button}, widget::{button, icon, Column, Element, Row}};

use super::Message;

fn plugin_reload_button<'a>(plugin: &Plugin) -> Element<'a, Message> {
  button(text("Reload"))
    .on_press(Message::Reload(plugin.info.name.clone()))
    .style(Button::Primary)
    .into()
}

fn plugin_details_state<'a>(plugin: &Plugin) -> Element<'a, Message> {
  let content: Element<_> = match &plugin.state {
    PluginState::Unloaded => text("The plugin is currently unloaded").into(),
    PluginState::Error(e) => column![
      text("The plugin has errored with the following error:"),
      text(format!("{:?}", e)),
    ].into(),
    PluginState::Loaded(_) => match plugin.enabled {
      true => text("The plugin is loaded and enabled").into(),
      false => text("The plugin is loaded but disabled").into(),
    }
  };

  content.into()
}

fn plugin_uninstall_button<'a>(plugin: &Plugin) -> Element<'a, Message> {
  button(text("Uninstall"))
  .on_press(Message::Uninstall(plugin.info.name.clone()))
  .style(Button::Destructive)
  .into()
}

pub fn plugin_details_view<'a>(plugin: &Plugin, show_reload_success_msg: bool) -> Element<'a, Message> {
  let reload_success_msg = match show_reload_success_msg {
    true => Some(text("Successfully reloaded")),
    false => None, 
  };

  column![
    container(
      column![
        row![
          button(icon(BootstrapIcon::ArrowLeft)).style(Button::Text).on_press(Message::GoBack),
          text(plugin.info.name.clone()).size(24),
        ].spacing(16).padding([0, 0, 8, 0]).align_items(Alignment::Center),
        row![
          text(plugin.info.version.clone()),
          text(format!("by {}", plugin.info.authors.join(", "))),
        ].spacing(8).padding([0, 0, 16, 0]),
        Row::new()
          .push(plugin_reload_button(plugin))
          .push_maybe(plugin_toggle_button(plugin))
          .push(plugin_uninstall_button(plugin))
          .push_maybe(reload_success_msg)
          .spacing(8)
          .padding([0, 0, 8, 0])
          .align_items(Alignment::Center),
        plugin_details_state(plugin),
      ]
    ).padding(8),
    container(rule::Rule::horizontal(1.0)).padding([0, 8, 0, 8]),
    plugin_details_content(plugin),
  ]
  .into()
}

fn plugin_description<'a>(description: String) -> Element<'a, Message> {
  let lines: Vec<Element<'a, Message>> = description
    .replace("\r\n", "\n")
    .split("\n")
    .map(|line| Into::<Element<'a, Message>>::into(text(line)))
    .collect();

  Column::from_vec(Vec::from_iter(lines))
    .spacing(6.0)
    .width(Length::Fill)
    .into()
}

fn plugin_details_content<'a>(plugin: &Plugin) -> Element<'a, Message> {
  let description = if plugin.info.description.len() > 0 {
    plugin.info.description.clone()
  } else {
    String::from("No description")
  };

  Scrollable::new(
    column![
      column![
        text("Description").size(24),
        plugin_description(description),
      ].spacing(8.0),

      column![
        text("Dependencies").size(24),
        dependencies_list(&plugin.info.dependencies),
      ]
    ]
    .spacing(24)
    .padding([8, 8, 8, 8])
  )
  .into()
}

fn dependencies_list<'a>(dependencies: &Vec<PluginDependency>) -> Element<'a, Message> {
  let mut list: Vec<Element<'a, Message>> = Vec::new();

  if dependencies.contains(&PluginDependency::Dangerous) {
    list.push(text("This plugin has a dangerous dependency. This means it is effectively able to escape the usual safety features. Make sure to audit the plugin.").style(theme::Text::Warn).into())
  }

  if dependencies.len() == 0 {
    return text("No dependencies").into();
  }

  for dependency in dependencies.iter() {
    list.push(text(format!("- {}", dependency)).into());
  }

  Column::<'a, Message>::from_vec(list).into()
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
      Toggler::new(
        String::from(label),
        enabled, 
        move |state| match state {
          true => Message::Enable(plugin_name.clone()),
          false => Message::Disable(plugin_name.clone()),
        }
    ).width(120)
  ).into()
  )
}