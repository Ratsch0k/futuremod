use futuremod_data::plugin::{Plugin, PluginDependency, PluginState};
use iced::{widget::{column, container, row, rule, text, Scrollable, Toggler}, Alignment, Length, Padding};
use iced_fonts::Bootstrap;

use crate::{theme::{self, Button}, widget::{button, icon, icon_text_button, icon_text_button_advanced, Column, Element, IconTextButtonOptions, Row}};

use super::Message;

fn plugin_reload_button<'a>(plugin: &Plugin) -> Element<'a, Message> {
  icon_text_button(Bootstrap::ArrowClockwise,"Reload")
    .on_press(Message::Reload(plugin.info.name.clone()))
    .class(Button::Primary)
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
  icon_text_button_advanced(Bootstrap::X,"Uninstall", IconTextButtonOptions::default().with_icon_size(24)).padding(Padding{top: 3.0, right: 16.0, bottom: 3.0, left: 8.0})
  .on_press(Message::UninstallPrompt(plugin.info.name.clone()))
  .class(Button::Destructive)
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
          button(icon(Bootstrap::ArrowLeft)).class(Button::Text).on_press(Message::GoBack),
          text(plugin.info.name.clone()).size(24),
        ].spacing(16).padding(Padding{top: 0.0, right: 0.0, bottom: 8.0, left: 0.0}).align_y(Alignment::Center),
        row![
          text(plugin.info.version.clone()),
          text(format!("by {}", plugin.info.authors.join(", "))),
        ].spacing(8).padding(Padding{top: 0.0, right: 0.0, bottom: 16.0, left: 0.0}),
        Row::new()
          .push(plugin_reload_button(plugin))
          .push_maybe(plugin_toggle_button(plugin))
          .push(plugin_uninstall_button(plugin))
          .push_maybe(reload_success_msg)
          .spacing(8)
          .padding(Padding{top: 0.0, right: 0.0, bottom: 8.0, left: 0.0})
          .align_y(Alignment::Center),
        plugin_details_state(plugin),
      ]
    ).padding(8),
    container(rule::Rule::horizontal(1.0)).padding(Padding{top: 0.0, right: 8.0, bottom: 0.0, left: 8.0}),
    plugin_details_content(plugin),
  ]
  .into()
}

fn plugin_description<'a>(description: String) -> Element<'a, Message> {
  let cleaned_description = description.replace("\r\n", "\n");
  let lines: Vec<String> = cleaned_description.split("\n").map(str::to_string).collect();

  let lines: Vec<Element<'a, Message>> = lines
    .into_iter()
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
    .padding(8)
  )
  .into()
}

fn dependencies_list<'a>(dependencies: &Vec<PluginDependency>) -> Element<'a, Message> {
  let mut list: Vec<Element<'a, Message>> = Vec::new();

  if dependencies.contains(&PluginDependency::Dangerous) {
    list.push(text("This plugin has a dangerous dependency. This means it is effectively able to escape the usual safety features. Make sure to audit the plugin.").class(theme::Text::Warn).into())
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