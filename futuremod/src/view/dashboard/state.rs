use iced::Command;
use log::{debug, info, warn};
use rfd::FileDialog;

use crate::{api::{self, get_plugins, reload_plugin}, util::{get_plugin_info_of_local_folder, is_plugin_folder}, view::{self, dashboard::view::{Dialog, InstallConfirmationPrompt}, logs}};

use super::{view::View, Dashboard, Message};

pub fn update(dashboard: &mut Dashboard, message: Message) -> Command<Message> {
  // Process some unique messages
  match message {
    Message::LogEvent(log_event) => dashboard.logs.handle_event(&log_event),
    Message::ResetView => {
      let plugins = dashboard.plugins.clone();
      *dashboard = Dashboard::new(plugins, dashboard.is_developer);
      return Command::none();
    },
    Message::Enable(plugin) | Message::Plugin(view::plugin::Message::Enable(plugin)) => {
      return Command::perform(async move {
        api::enable_plugin(plugin).await.map_err(|e| e.to_string())
      }, Message::EnableResponse);
    },
    Message::Disable(plugin) | Message::Plugin(view::plugin::Message::Disable(plugin)) => {
      return Command::perform(async {
        api::disable_plugin(plugin).await.map_err(|e| e.to_string())
      }, Message::DisableResponse);
    },
    Message::EnableResponse(response) => {
      match response {
        Ok(()) => {
          return Command::perform(api::get_plugins(), Message::GetPluginsResponse);
        },
        Err(e) => {
          dashboard.dialog = Some(Dialog::Error(format!("Could not enable the plugin: {}", e).to_string()));
          warn!("Could not enable plugin: {}", e);
        }
      }
    },
    Message::DisableResponse(response) => {
      match response {
        Ok(()) => {
          return Command::perform(api::get_plugins(), Message::GetPluginsResponse);
        },
        Err(e) => {
          dashboard.dialog = Some(Dialog::Error(format!("Could not disable the plugin: {}", e).to_string()));
          warn!("Could not disable plugin: {}", e);
        }
      }
    },
    Message::GetPluginsResponse(response) => {
      match response {
        Ok(plugins) => {
          dashboard.plugins = plugins;
        },
        Err(e) => {
          warn!("Could not get plugins: {}", e);
        }
      }
    },
    Message::UninstallPrompt(plugin_name) | Message::Plugin(view::plugin::Message::UninstallPrompt(plugin_name)) => {
      dashboard.dialog = Some(Dialog::UninstallPrompt(plugin_name));
    },
    Message::Uninstall(name) => {
      return Command::perform(async move {api::uninstall_plugin(name).await.map_err(|e| e.to_string())}, Message::UninstallResponse);
    },
    Message::UninstallResponse(response) => {
      match response {
        Ok(()) => {
          // Navigate back to plugin list
          dashboard.view = None;
          dashboard.dialog = None;
          return Command::perform(get_plugins(), Message::GetPluginsResponse)
        },
        Err(e) => {
          warn!("Could not uninstall error: {}", e);
          dashboard.dialog = Some(Dialog::Error(format!("Could not uninstall the plugin: {}", e).to_string()));
        }
      }
    },
    Message::Reload(name) | Message::Plugin(view::plugin::Message::Reload(name)) => {
      return Command::perform(async move {reload_plugin(&name).await.map_err(|e| format!("{}", e))}, Message::ReloadResponse);
    },
    Message::ReloadResponse(response) => {
      match response {
        Ok(()) => return Command::perform(get_plugins(), Message::GetPluginsResponse),
        Err(e) => {
          warn!("Could not reload plugin: {}", e);
          dashboard.dialog = Some(Dialog::Error(format!("Could not reload the plugin: {}", e).to_string()));
        },
      }
    }
    Message::ToPlugins => {
      dashboard.view = None;
    },
    Message::StartInstallation => {
      let plugin_package = match FileDialog::new()
        .set_title("Select the Plugin Package to install")
        .add_filter("Plugin Package", &["zip"])
        .pick_file() {
          Some(v) => v,
          None => return Command::none(),
      };

      info!("Get plugin info of plugin package at '{}'", plugin_package.display());

      return Command::perform(async {
        let response = api::get_plugin_info(plugin_package.clone()).await.map_err(|e| e.to_string())?;

        Ok(InstallConfirmationPrompt {
          plugin: response,
          path: plugin_package,
          in_developer_mode: false,
        })
      }, Message::OpenInstallConfirmationPromptDialog);
    },
    Message::StartDevelopmentInstallation => {
      let plugin_package = match FileDialog::new()
        .set_title("Select the Plugin Directory to install in development mode")
        .pick_folder() {
          Some(v) => v,
          None => return Command::none(),
      };

      info!("Selected plugin folder at '{}'", plugin_package.display());

      debug!("Check if the selected directory contains a plugin");
      match is_plugin_folder(&plugin_package) {
        Ok(false) => {
          warn!("The directory does not contain a valid plugin");
          return Command::none();
        },
        Err(e) => {
          warn!("Could not check the folder: {}", e);
          
          return Command::none()
        },
        _ => (),
      };

      info!("Get plugin info of plugin package at '{}'", plugin_package.display());

      return Command::perform(async move {
        let response = get_plugin_info_of_local_folder(&plugin_package).map_err(|e| e.to_string())?;

        Ok(InstallConfirmationPrompt {
          plugin: response,
          path: plugin_package,
          in_developer_mode: true,
        })
      }, Message::OpenInstallConfirmationPromptDialog);
    },
    Message::OpenInstallConfirmationPromptDialog(prompt_response) => {
      match prompt_response {
        Ok(prompt) => {
          dashboard.dialog = Some(Dialog::InstallationPrompt(prompt));
        },
        Err(e) => {
          warn!("Could not fetch plugin information: {}", e);
          dashboard.dialog = Some(Dialog::Error(format!("Could not fetch plugin information: {}", e).to_string()));
        },
      }
    },
    Message::ConfirmInstallation(confirmed_prompt) => {
      info!("Install plugin package at '{}'", confirmed_prompt.path.display());

      return Command::perform(async move {
        match confirmed_prompt.in_developer_mode {
          false => api::install_plugin(&confirmed_prompt.path).await.map_err(|e| e.to_string()),
          true => api::install_plugin_in_developer_mode(&confirmed_prompt.path).await.map_err(|e| e.to_string()),
        }
      }, Message::InstallResponse);
    },
    Message::InstallResponse(response) => {
      match response {
        Ok(()) => {
          return Command::perform(async {get_plugins().await.map_err(|e| e.to_string())}, Message::InstallGetPlugins);
        },
        Err(e) => {
          warn!("Could not install plugin: {}", e);
          dashboard.dialog = Some(Dialog::Error(format!("Could not install plugin: {}", e).to_string()));
        }
      }
    },
    Message::InstallGetPlugins(response) => {
      dashboard.dialog = None;
      return Command::perform(async {}, |_| Message::GetPluginsResponse(response));
    },
    Message::CloseDialog => {
      dashboard.dialog = None;
    },
    Message::ToLogs => {
      let (logs_view, logs_message) = logs::Logs::new();

      dashboard.view = Some(View::Logs(logs_view));

      return logs_message.map(Message::Logs);
    },
    Message::ToPlugin(name) => {
      let plugin = dashboard.plugins.get(&name);
      match plugin {
        Some(_) => {
          dashboard.view = Some(View::Plugin(view::plugin::Plugin::new(name.clone())));
        },
        None => {
        }
      }
    },
    // Message decision tree based on view state
    message => match &mut dashboard.view {
      Some(view) => match view {
        View::Logs(logs_view) => match message {
          Message::Logs(logs_message) => match logs_message {
            other_logs_message => {
              return logs_view.update(other_logs_message).map(Message::Logs)
            },
          },
          _ => (),
        },
        View::Plugin(_) => match message {
          Message::Plugin(plugin_message) => match plugin_message {
            view::plugin::Message::GoBack => {
              dashboard.view = None;
            }
            _ => (),
          },
          _ => (),
        }
      },
      _ => (),
    },
  }

  Command::none()
}