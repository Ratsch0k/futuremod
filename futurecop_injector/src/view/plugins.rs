use std::{collections::HashMap, path::PathBuf};

use iced::{alignment::Vertical, futures::TryFutureExt, widget::{column, container, row, rule, scrollable, text, Space, Toggler}, Alignment, Command, Length, Padding};
use iced_aw::{modal, BootstrapIcon};
use log::{info, warn};
use rfd::FileDialog;
use futurecop_data::plugin::*;

use crate::{api::{build_url, get_plugin_info, get_plugins, install_plugin, reload_plugin, uninstall_plugin}, theme::{self, Container, Text, Theme}, util::wait_for_ms, widget::{button, icon, icon_with_style, Column, Element, Row}};
use crate::theme::Button;

#[derive(Debug, Clone)]
pub struct PluginsView {
  plugins: HashMap<String, Plugin>,
  selected_plugin: Option<String>,
  error: Option<String>,
  confirm_installation: Option<InstallConfirmationPrompt>,
  show_reload_success_message: bool,
}

#[derive(Debug, Clone)]
pub enum Plugins {
  Error(String),
  Loading,
  Loaded(PluginsView)
}

#[derive(Debug, Clone)]
pub struct InstallConfirmationPrompt {
  pub plugin: PluginInfo,
  pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum Message {
  GetPluginsResult(Result<HashMap<String, Plugin>, String>),
  Enable(String),
  EnableResponse(Option<String>),
  Disable(String),
  DisableResponse(Option<String>),
  Reload(String),
  ReloadResponse(Result<HashMap<String, Plugin>, String>),
  GoToDetails(String),
  GoToOverview,
  GoBack,
  SelectPluginToInstall,
  PluginInfoResponse(Result<InstallConfirmationPrompt, String>),
  ConfirmInstallation(InstallConfirmationPrompt),
  CancelInstallation,
  InstallResponse(Result<(), String>),
  ClearError,
  UninstallPlugin(String),
  UninstallPluginResponse(Result<String, String>),
  HideReloadSuccessfulMessage,
}


impl Plugins {
  pub fn new() -> (Self, Command<Message>) {
    (
      Plugins::Loading,
      Command::perform(get_plugins(), Message::GetPluginsResult)
    )
  }

  pub fn update(&mut self, message: Message) -> iced::Command<Message> {
      match self {
        Plugins::Loading => match message {
          Message::GetPluginsResult(result) => match result {
              Ok(result) => {
                *self = Plugins::Loaded(PluginsView{
                  plugins: result,
                  selected_plugin: None, 
                  error: None, 
                  confirm_installation: None, 
                  show_reload_success_message: false
                });
                Command::none()
              },
              Err(e) => {
                *self = Plugins::Error(e);
                Command::none()
              },
          },
          _ => Command::none(),
        }
        Plugins::Error(_) => todo!(),
        Plugins::Loaded(plugins_view) => match message {
          Message::GetPluginsResult(result) => match result {
              Ok(result) => {
                plugins_view.plugins = result;
                Command::none()
              },
              Err(e) => {
                *self = Plugins::Error(e);
                Command::none()
              },
          },
          Message::Enable(name) => Command::perform(enable_plugin(name), Message::EnableResponse),
          Message::Disable(name) => Command::perform(disable_plugin(name), Message::DisableResponse),
          Message::DisableResponse(response) => match response {
            Some(name) => {
              match plugins_view.plugins.get_mut(&name) {
                Some(plugin) => {
                  plugin.enabled = false;

                  Command::none()
                },
                None => Command::none(),
              }
            },
            None => Command::none(),
          },
          Message::EnableResponse(response) => match response {
            Some(name) => {
              match plugins_view.plugins.get_mut(&name) {
                Some(plugin) => {
                  plugin.enabled = true;

                  Command::none()
                },
                None => Command::none(),
              }
            },
            None => Command::none(),
          },
          Message::GoToDetails(name) => {
            plugins_view.selected_plugin = Some(name);
            Command::none()
          },
          Message::GoToOverview => {
            plugins_view.selected_plugin = None;
            Command::none()
          }
          Message::Reload(plugin_name) => {
            Command::perform(reload_and_get_plugins(plugin_name.clone()), Message::ReloadResponse)
          },
          Message::ReloadResponse(response) => {
            match response {
              Ok(new_plugins) => {
                plugins_view.plugins = new_plugins;
                plugins_view.show_reload_success_message = true;

                Command::perform(
                  wait_for_ms(3000), 
                  |_| Message::HideReloadSuccessfulMessage,
                )
              },
              Err(e) => {
                *self = Plugins::Error(e);


                Command::none()
              }
            }
          },
          Message::HideReloadSuccessfulMessage => {
            plugins_view.show_reload_success_message = false;

            Command::none()
          }
          Message::SelectPluginToInstall => {
            let plugin_package = match FileDialog::new()
              .set_title("Select the Plugin Package to install")
              .add_filter("Plugin Package", &["zip"])
              .pick_file() {
                Some(v) => v,
                None => return Command::none(),
            };

            info!("Get plugin info of plugin package at '{}'", plugin_package.display());

            Command::perform(async {
              let response = get_plugin_info(plugin_package.clone()).map_err(|e| e.to_string()).await?;

              Ok(InstallConfirmationPrompt {
                plugin: response,
                path: plugin_package,
              })
            }, Message::PluginInfoResponse)
          },
          Message::PluginInfoResponse(result) => match result {
            Ok(info) => {
              plugins_view.confirm_installation = Some(info);
              Command::none()
            },
            Err(e) => {
              plugins_view.error = Some(e);
              Command::none()
            }
          },
          Message::ConfirmInstallation(confirmation) => {
            info!("Install plugin package at '{}'", confirmation.path.display());

            Command::perform(install_plugin(confirmation.path).map_err(|e| e.to_string()), Message::InstallResponse)
          },
          Message::CancelInstallation => {
            plugins_view.confirm_installation = None;

            Command::none()
          },
          Message::InstallResponse(result) => {
            plugins_view.confirm_installation = None;

            match result {
              Ok(()) => {
                info!("Successfully installed plugin, reloading plugin list");


                Command::perform(get_plugins(), Message::GetPluginsResult)
              },
              Err(err) => {
                warn!("Could not install plugin: {}", err);
                plugins_view.error = Some(err);

                Command::perform(get_plugins(), Message::GetPluginsResult)
              }
            }
          },
          Message::ClearError => {
            info!("Clearing error");
            plugins_view.error = None;

            Command::none()
          },
          Message::UninstallPlugin(plugin_name) => {
            info!("Uninstalling plugin '{}'", plugin_name);

            Command::perform(async {
              uninstall_plugin(plugin_name.clone()).await.map_err(|e| e.to_string())?;
              Ok(plugin_name)
            }, Message::UninstallPluginResponse)
          },
          Message::UninstallPluginResponse(result) => {
            match result {
              Ok(name) => {
                info!("Successfully uninstalled plugin '{}'", name);

                let _ = plugins_view.plugins.remove(&name);
                if plugins_view.selected_plugin.as_ref().is_some_and(|v| *v == name) {
                  plugins_view.selected_plugin = None;
                }
              },
              Err(err) => {
                warn!("Could not uninstall plugin: {}", err);
                plugins_view.error = Some(err);
              }
            }

            Command::none()
          }
          _ => Command::none(),
        },
      }
  }

  pub fn view(&self) -> Element<Message> {
      match self {
          Plugins::Error(e) => {
            text(format!("Could not get plugins: {}", e))
            .into()
          },
          Plugins::Loading => {
            text("Loading plugins...")
            .into()
          },
          Plugins::Loaded(plugin_view) => {
            if let Some(plugin_name) = &plugin_view.selected_plugin {
              let plugin = plugin_view.plugins.get(plugin_name).unwrap();

              return plugin_details_view(plugin, plugin_view.show_reload_success_message);
            }

            let mut list = Column::new();

            for (name, plugin) in plugin_view.plugins.iter() {
              list = list.push(plugin_card(name, plugin));
            }

            list = list
              .spacing(12)
              .padding(Padding::new(24.0))
              .height(Length::Fill)
              .width(Length::Fill);

            let mut content = column![
              container(
                row![
                  button(icon(iced_aw::BootstrapIcon::ArrowLeft)).style(Button::Text).on_press(Message::GoBack),
                  container(text("Plugins").size(24).vertical_alignment(Vertical::Center)).width(Length::Fill).align_y(Vertical::Center),
                  button("Install Plugin").on_press(Message::SelectPluginToInstall).style(Button::Primary)
                ]
                  .spacing(16)
                  .align_items(iced::Alignment::Center),
              ).padding(8),  
            ];

            if let Some(err) = &plugin_view.error {
              content = content.push(
                container(
                    container(
                      row![
                        text(err).width(Length::Fill),
                        button(icon_with_style(BootstrapIcon::X, Text::Danger)).on_press(Message::ClearError).style(Button::Text)
                      ].align_items(iced::Alignment::Center),
                    )
                    .padding(16)
                    .style(Container::Danger)
                  )
                  .padding(16)
              )
            }

            let underlay: Element<'_, Message> = content
              .push(list)
              .into();

            let overlay = if let Some(confirmation_prompt) = &plugin_view.confirm_installation {
              let warning: Option<iced::widget::Container<Message, Theme>> = if confirmation_prompt.plugin.dependencies.contains(&PluginDependency::Dangerous) {
                Some(
                  container(
                    text("This plugin has a dangerous dependency. This plugin can easily access your entire computer. Only install plugins with dangerous dependency if you are sure they are not malicious.")
                  )
                  .style(Container::Warning)
                  .padding(8)
                )
              } else {
                None
              };

              Some(
                container(
                  column![
                  text("Confirm installation").size(24.0),
                  Space::with_height(12.0),
                  scrollable(
                    Column::new()
                      .push(text(format!("Are you sure you want to install the plugin '{}'.", confirmation_prompt.plugin.name.clone())))
                      .push_maybe(warning)
                      .push(column![
                        text("General Information").size(24),
                        text(format!("Name: {}", confirmation_prompt.plugin.name.clone())),
                        text(format!("Authors: {}", confirmation_prompt.plugin.authors.clone().join(", "))),
                        text(format!("Version: {}", confirmation_prompt.plugin.version)),
                      ].spacing(4))
                      .push(column![
                        text("Description").size(24),
                        text(
                          if confirmation_prompt.plugin.description.len() <= 0 {
                            String::from("No description")
                          } else {
                            confirmation_prompt.plugin.description.clone()
                          }
                        ),
                      ].spacing(4))
                      .push(column![
                        text("Dependencies").size(24),
                        dependencies_list(&confirmation_prompt.plugin.dependencies),
                      ].spacing(4))
                      .spacing(24),
                  ),
                  row![
                    Space::with_width(Length::Fill),
                    button(text("Cancel")).style(Button::Destructive).on_press(Message::CancelInstallation),
                    button(text("Install")).on_press(Message::ConfirmInstallation(confirmation_prompt.clone())).style(Button::Primary),
                  ]
                  .align_items(Alignment::End)
                  .spacing(8.0)
                  .width(Length::Fill)
                  ])
                  .max_width(500.0)
                  .style(Container::Dialog)
                  .padding(16.0)
              )
            } else {
              None
            };

            modal(underlay, overlay)
              .backdrop(Message::CancelInstallation)
              .on_esc(Message::CancelInstallation)
              .into()
          },
      }
  }
}

fn plugin_card<'a>(name: &String, plugin: &Plugin) -> Element<'a, Message> {
  container(
    row![
      Column::new()
        .push(text(name).size(20))
        .push(plugin_state_component(plugin))
        .width(Length::Fill),
      Row::new()
      .push(plugin_go_to_details_button(plugin))
      .push_maybe(plugin_toggle_button(plugin))
      .spacing(8)
      .align_items(Alignment::Center)
    ]
    .align_items(Alignment::Center)
  )
  .style(Container::Box)
  .padding(16)
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
    .size(12)
    .into()
}

fn plugin_go_to_details_button<'a>(plugin: &Plugin) -> Element<'a, Message> {
  button(text("Details"))
    .on_press(Message::GoToDetails(plugin.info.name.clone()))
    .style(Button::Default)
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
  .on_press(Message::UninstallPlugin(plugin.info.name.clone()))
  .style(Button::Destructive)
  .into()
}

fn plugin_details_view<'a>(plugin: &Plugin, show_reload_success_msg: bool) -> Element<'a, Message> {
  let reload_success_msg = match show_reload_success_msg {
    true => Some(text("Successfully reloaded")),
    false => None, 
  };

  column![
    row![
      button(icon(BootstrapIcon::ArrowLeft)).style(Button::Text).on_press(Message::GoToOverview),
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
    container(rule::Rule::horizontal(1.0)).padding([8.0, 0.0, 8.0, 0.0]),
    plugin_details_content(plugin),
  ]
  .padding(8)
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
    .into()
}

fn plugin_details_content<'a>(plugin: &Plugin) -> Element<'a, Message> {
  let description = if plugin.info.description.len() > 0 {
    plugin.info.description.clone()
  } else {
    String::from("No description")
  };

  column![
    column![
      text("Description").size(24),
      plugin_description(description),
    ].spacing(8.0),

    column![
      text("Dependencies").size(24),
      dependencies_list(&plugin.info.dependencies),
    ]
  ].spacing(24)
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

async fn enable_plugin(name: String) -> Option<String> {
  let mut body = HashMap::new();
  body.insert("name", name.clone());

  match reqwest::Client::new()
    .put(build_url("/plugin/enable"))
    .json(&body)
    .send()
    .await {
        Ok(_) => Some(name),
        Err(_) => None,
    }
}

async fn disable_plugin(name: String) -> Option<String> {
  let mut body = HashMap::new();
  body.insert("name", name.clone());

  match reqwest::Client::new()
    .put(build_url("/plugin/disable"))
    .json(&body)
    .send()
    .await {
        Ok(_) => Some(name),
        Err(_) => None,
    }
}

async fn reload_and_get_plugins(name: String) -> Result<HashMap<String, Plugin>, String> {
  match reload_plugin(name.as_str()).await {
    Err(e) => return Err(format!("{:?}", e)),
    _ => (),
  };

  get_plugins().await
}