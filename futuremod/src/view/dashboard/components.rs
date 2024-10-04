use std::collections::HashMap;

use futuremod_data::plugin::{Plugin, PluginDependency, PluginState};
use iced::{alignment::{Horizontal, Vertical}, widget::{center, column, container, mouse_area, opaque, row, rule, scrollable, text, Space, Stack, Toggler}, Alignment, Length, Padding};
use iced_aw::Badge;
use iced_fonts::Bootstrap;
use log::warn;

use crate::{theme::{self, Container, Theme}, widget::{bold, button, icon_button, icon_text_button, icon_text_button_advanced, Column, Element, IconTextButtonOptions, Row}};

use super::{view::{Dialog, InstallConfirmationPrompt, View}, Dashboard, Message};

pub fn dashboard<'a>(state: &'a Dashboard) -> Element<'a, Message> {
  let content = match &state.view {
    None => plugin_overview(&state.plugins, state.is_developer),
    Some(view) => match view {
      View::Logs(logs_view) => logs_view.view(&state.logs).map(Message::Logs),
      View::Plugin(plugin_view) => {
        match state.plugins.get(&plugin_view.name) {
          Some(plugin) => plugin_view.view(&plugin).map(Message::Plugin),
          None => error_box("Plugin doesn't exist".to_string()),
        }
      }
    },
  };

  let underlay: Element<Message> = column![
    heading(state.is_developer, &state.view),
    rule::Rule::horizontal(1.0),
    Space::new(Length::Fill, 8.0),
    content
  ].into();

  let mut overlay: Option<Element<Message>> = None;
  if let Some(active_dialog) = &state.dialog {
    overlay = Some(
      opaque(
        mouse_area(
          center(
            opaque(
                dialog(active_dialog)
              )
            )
              .class(Container::Backdrop)
        )
          .on_press(Message::CloseDialog)
      )
    );
  }

  Stack::with_children([underlay])
    .push_maybe(overlay)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn dialog<'a>(active_dialog: &'a Dialog) -> Element<'a, Message> {
  match active_dialog {
    Dialog::InstallationPrompt(prompt) => installation_prompt(prompt),
    Dialog::Error(error) => error_dialog(error),
    Dialog::UninstallPrompt(plugin_name) => uninstall_prompt(plugin_name.clone()),
  }
}

fn uninstall_prompt<'a>(plugin_name: String) -> Element<'a, Message> {
  container(
    column![
      dialog_header(format!("Uninstall {}", plugin_name)),
      Space::with_height(16),
      text("Uninstalling a plugin cannot be undone. Are you sure?"),
      Space::with_height(24),
      row![
        Space::with_width(Length::Fill),
        button("Cancel").on_press(Message::CloseDialog),
        button("Uninstall").on_press(Message::Uninstall(plugin_name)).class(theme::Button::Destructive),
      ]
        .spacing(8),
    ]
  )
    .class(Container::Dialog)
    .padding(16)
    .max_width(500)
    .into()
}

fn error_dialog<'a>(error: &'a String) -> Element<'a, Message> {
  container(
    column![
      dialog_header(String::from("Error")),
      Space::with_height(16),
      text(error),
    ]
  )
    .class(Container::Dialog)
    .padding(16)
    .max_width(500)
    .into()
}

fn dialog_header<'a>(title: String) -> Element<'a, Message> {
  row![
    container(text(title).size(24)).width(Length::Fill),
    icon_button(Bootstrap::X).on_press(Message::CloseDialog).class(theme::Button::Text),
  ]
    .align_y(Alignment::Center)
    .into()
}

fn heading<'a>(is_developer: bool, active_view: &Option<View>) -> Element<'a, Message> {
  row![
    container(title(is_developer)).width(Length::Fill).padding(Padding{top: 0.0, right: 0.0, bottom: 0.0, left: 8.0}),
    container(
      tabs(active_view)
    )
      .padding(8.0),
  ]
    .align_y(Alignment::Center)
    .into()
}

fn title<'a>(is_developer: bool) -> Element<'a, Message> {
  let content = match is_developer {
    true => "FutureMod - Developer",
    false => "FutureMod",
  };

  text(content).size(32.0).into()
}

fn tabs<'a>(active_view: &Option<View>) -> Element<'a, Message> {
  let is_plugin_tab = |active_view: &Option<View>| {
    match active_view {
      None | Some(View::Plugin(_)) => true,
      _ => false,
    }
  };

  row![
    icon_text_button(Bootstrap::Box, "Plugins")
      .on_press(Message::ToPlugins)
      .class(if is_plugin_tab(&active_view) {theme::Button::Primary} else {theme::Button::HoverHighlight}),
    icon_text_button(Bootstrap::CardText, "Logs")
      .on_press(Message::ToLogs)
      .class(if let Some(View::Logs(_)) = active_view {theme::Button::Primary} else {theme::Button::HoverHighlight}),
    icon_text_button(Bootstrap::Gear, "Settings")
      .on_press(Message::ToSettings),
  ]
    .spacing(8.0)
    .into()
}

fn plugin_overview<'a>(plugins: &'a HashMap<String, Plugin>, is_developer: bool) -> Element<'a, Message> {
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
        .on_press(Message::StartDevelopmentInstallation)
    );
  }

  actions = actions.push(
    icon_text_button_advanced(Bootstrap::Plus, "Install", IconTextButtonOptions::new().with_icon_size(24))
      .class(theme::Button::Primary)
      .padding(Padding{top: 3.0, right: 16.0, bottom: 3.0, left: 12.0})
      .on_press(Message::StartInstallation)
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
  .class(Container::Box)
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

pub fn error_box<'a>(message: String) -> Element<'a, Message> {
  container(
    column![
      text(message).size(24),
      button("Reset").on_press(Message::ResetView)
    ]
      .spacing(8.0)
  )
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .padding(8.0)
    .into()
}

fn installation_prompt<'a>(confirmation_prompt: &InstallConfirmationPrompt) -> Element<'a, Message> {
  let warning: Option<iced::widget::Container<Message, Theme>> = if confirmation_prompt.plugin.dependencies.contains(&PluginDependency::Dangerous) {
    Some(
      container(
        text("This plugin has a dangerous dependency. This plugin can easily access your entire computer. Only install plugins with dangerous dependency if you are sure they are not malicious.")
      )
      .class(Container::Warning)
      .padding(8)
    )
  } else {
    None
  };

  container(
    container(
      column![
        text("Confirm installation").size(24.0),
        Space::with_height(12.0),
        container(
          scrollable(
            Column::new()
              .push(text(format!("Are you sure you want to install the plugin '{}'.", confirmation_prompt.plugin.name.clone())))
              .push_maybe(warning)
              .push(column![
                  text("General Information").size(20),
                  text(format!("Name: {}", confirmation_prompt.plugin.name.clone())),
                  text(format!("Authors: {}", confirmation_prompt.plugin.authors.clone().join(", "))),
                  text(format!("Version: {}", confirmation_prompt.plugin.version)),
                ]
                  .spacing(4))
                  .push(column![
                    text("Description").size(20),
                    text(
                      if confirmation_prompt.plugin.description.len() <= 0 {
                        String::from("No description")
                      } else {
                        confirmation_prompt.plugin.description.clone()
                      }
                    ),
                  ]
                    .spacing(4))
                    .push(column![
                      text("Dependencies").size(20),
                      dependencies_list(&confirmation_prompt.plugin.dependencies),
                    ].spacing(4))
                    .spacing(24)
                    .padding(Padding{top: 0.0, right: 16.0, bottom: 0.0, left: 8.0}),
          )
        )
          .height(Length::Fill)
          .padding(Padding{top: 0.0, right: 0.0, bottom: 8.0, left: 0.0}),
        row![
          Space::with_width(Length::Fill),
          button(text("Cancel")).class(theme::Button::Destructive).on_press(Message::CloseDialog),
          button(text("Install")).on_press(Message::ConfirmInstallation(confirmation_prompt.clone())).class(theme::Button::Primary),
        ]
          .align_y(Alignment::End)
          .spacing(8.0)
          .width(Length::Fill)
          .height(Length::Fixed(60.0))
      ]
    )
      .height(Length::Shrink)
      .max_width(800.0)
      .class(Container::Dialog)
      .padding(16.0)
  )
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .padding(8.0)
    .into()
}