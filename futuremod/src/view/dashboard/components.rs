use std::time::Instant;

use futuremod_data::plugin::PluginDependency;
use iced::{alignment::{Horizontal, Vertical}, widget::{center, column, container, mouse_area, opaque, row, rule, scrollable, text, Space, Stack}, Alignment, Color, Length, Padding};
use iced_fonts::Bootstrap;
use lilt::Animated;

use crate::{theme::{self, Container, Theme}, widget::{button, icon_button, icon_with_size, Column, Element, Row}};

use super::{view::{Dialog, InstallConfirmationPrompt, View}, Dashboard, Message};

pub fn dashboard<'a>(state: &'a Dashboard) -> Element<'a, Message> {
  let content = match &state.view {
    View::Logs(logs_view) => logs_view.view(&state.logs).map(Message::Logs),
    View::Plugin(plugin_view) => {
      match state.plugins.get(&plugin_view.name) {
        Some(plugin) => plugin_view.view(&plugin).map(Message::Plugin),
        None => error_box("Plugin doesn't exist".to_string()),
      }
    },
    View::PluginList(plugin_list_view) => plugin_list_view.view(&state.plugins, state.is_developer)
      .map(Message::PluginList),
    View::Settings(settings_view) => settings_view.view().map(Message::Settings),
  };

  let underlay: Element<Message> = column![
    heading(state.is_developer),
    rule::Rule::horizontal(1.0),
    row![
      sidebar(&state.view, &state.sidebar_minimized),
      rule::Rule::vertical(1.0),
      content,
    ]
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

fn sidebar<'a>(active_view: &'a View, minimized: &'a Animated<bool, Instant>) -> Element<'a, Message> {
  container(
    tabs(active_view, &minimized)
  )
    .padding(16)
    .width(minimized.animate_bool(200.0, 80.0, Instant::now()))
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

fn heading<'a>(is_developer: bool) -> Element<'a, Message> {
  row![
    container(title(is_developer)).width(Length::Fill).padding(Padding{top: 0.0, right: 0.0, bottom: 0.0, left: 8.0}),
  ]
    .padding(8)
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

fn tabs<'a>(active_view: &View, minimized: &Animated<bool, Instant>) -> Element<'a, Message> {
  let is_plugin_tab = |active_view: &View| {
    match active_view {
      View::PluginList(_) | View::Plugin(_) => true,
      _ => false,
    }
  };

  let tab_button = |icon_content: Bootstrap, label: &'static str, on_press: Option<Message>, active: bool| -> Element<'_, Message> {
    let mut content = Row::with_children([icon_with_size(icon_content, 16).into()])
      .clip(false);

    let alpha = minimized.animate_bool(1.0, 0.0, Instant::now());

    if alpha > 0.1 {
      content = content.push(
        container(
          text(label)
            .wrapping(text::Wrapping::None)
            .height(20)
            .class(theme::Text::Custom(Box::new(move |theme| text::Style {
              color: Some(Color{
                a: alpha,
                ..theme.palette.background.dark.text
              }),
            })))
        )
          .padding(Padding::default().left(8))
      );
    }

    button(content)
      .on_press_maybe(on_press)
      .class(if active {theme::Button::Primary} else {theme::Button::Text})
      .width(Length::Fill)
      .clip(false)
      .into()
  };

  column![
    tab_button(if !minimized.value {Bootstrap::ChevronDoubleLeft} else {Bootstrap::ChevronDoubleRight}, "Minimize", Some(Message::ToggleSidebar), false),
    tab_button(Bootstrap::Box, "Plugins", Some(Message::ToPluginList), is_plugin_tab(&active_view)),
    tab_button(Bootstrap::CardText, "Logs", Some(Message::ToLogs), matches!(active_view, View::Logs(_))),
    Space::with_height(Length::Fill),
    tab_button(Bootstrap::Gear, "Settings", Some(Message::ToSettings), false),
  ]
    .spacing(8.0)
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
      ]
    )
      .max_width(800.0)
      .class(Container::Dialog)
      .padding(16.0)
  )
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .padding(8.0)
    .into()
}