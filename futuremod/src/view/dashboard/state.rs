use std::time::Instant;

use iced::Task;
use log::{debug, info, warn};
use rfd::FileDialog;

use crate::{api::{self, get_plugins, reload_plugin}, util::{get_plugin_info_of_local_folder, is_plugin_folder}, view::{self, dashboard::view::{Dialog, InstallConfirmationPrompt}, logs}};

use super::{view::View, Dashboard, Message};

pub fn update(dashboard: &mut Dashboard, message: Message) -> Task<Message> {
  // Process some unique messages
  match message {
    Message::LogEvent(log_event) => dashboard.logs.handle_event(&log_event),
    Message::ResetView => {
      let plugins = dashboard.plugins.clone();
      *dashboard = Dashboard::new(plugins, dashboard.is_developer);
      return Task::none();
    },
    Message::Plugin(view::plugin::Message::Enable(plugin)) |
    Message::PluginList(view::plugin_list::Message::Enable(plugin)) => {
      return Task::perform(async move {
        api::enable_plugin(plugin).await.map_err(|e| e.to_string())
      }, Message::EnableResponse);
    },
    Message::Plugin(view::plugin::Message::Disable(plugin)) |
    Message::PluginList(view::plugin_list::Message::Disable(plugin)) => {
      return Task::perform(async {
        api::disable_plugin(plugin).await.map_err(|e| e.to_string())
      }, Message::DisableResponse);
    },
    Message::EnableResponse(response) => {
      match response {
        Ok(()) => {
          return Task::perform(api::get_plugins(), Message::GetPluginsResponse);
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
          return Task::perform(api::get_plugins(), Message::GetPluginsResponse);
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
    Message::UninstallPrompt(plugin_name) |
    Message::Plugin(view::plugin::Message::UninstallPrompt(plugin_name)) => {
      dashboard.dialog = Some(Dialog::UninstallPrompt(plugin_name));
    },
    Message::Uninstall(name) => {
      return Task::perform(async move {api::uninstall_plugin(name).await.map_err(|e| e.to_string())}, Message::UninstallResponse);
    },
    Message::UninstallResponse(response) => {
      match response {
        Ok(()) => {
          // Navigate back to plugin list
          return Task::perform(get_plugins(), Message::GetPluginsResponse)
            .chain(
              Task::batch(
                [
                    Task::done(Message::CloseDialog),
                    Task::done(Message::ToPluginList)
                  ]
                )
            );
        },
        Err(e) => {
          warn!("Could not uninstall error: {}", e);
          dashboard.dialog = Some(Dialog::Error(format!("Could not uninstall the plugin: {}", e).to_string()));
        }
      }
    },
    Message::Reload(name) | Message::Plugin(view::plugin::Message::Reload(name)) => {
      return Task::perform(async move {reload_plugin(&name).await.map_err(|e| format!("{}", e))}, Message::ReloadResponse);
    },
    Message::ReloadResponse(response) => {
      match response {
        Ok(()) => return Task::perform(get_plugins(), Message::GetPluginsResponse),
        Err(e) => {
          warn!("Could not reload plugin: {}", e);
          dashboard.dialog = Some(Dialog::Error(format!("Could not reload the plugin: {}", e).to_string()));
        },
      }
    }
    Message::ToPluginList => {
      dashboard.view = View::PluginList(view::plugin_list::PluginList::new());
    },
    Message::PluginList(view::plugin_list::Message::Install) => {
      let plugin_package = match FileDialog::new()
        .set_title("Select the Plugin Package to install")
        .add_filter("Plugin Package", &["zip"])
        .pick_file() {
          Some(v) => v,
          None => return Task::none(),
      };

      info!("Get plugin info of plugin package at '{}'", plugin_package.display());

      return Task::perform(async {
        let response = api::get_plugin_info(plugin_package.clone()).await.map_err(|e| e.to_string())?;

        Ok(InstallConfirmationPrompt {
          plugin: response,
          path: plugin_package,
          in_developer_mode: false,
        })
      }, Message::OpenInstallConfirmationPromptDialog);
    },
    Message::PluginList(view::plugin_list::Message::DevInstall) => {
      let plugin_package = match FileDialog::new()
        .set_title("Select the Plugin Directory to install in development mode")
        .pick_folder() {
          Some(v) => v,
          None => return Task::none(),
      };

      info!("Selected plugin folder at '{}'", plugin_package.display());

      debug!("Check if the selected directory contains a plugin");
      match is_plugin_folder(&plugin_package) {
        Ok(false) => {
          warn!("The directory does not contain a valid plugin");
          return Task::none();
        },
        Err(e) => {
          warn!("Could not check the folder: {}", e);
          
          return Task::none()
        },
        _ => (),
      };

      info!("Get plugin info of plugin package at '{}'", plugin_package.display());

      return Task::perform(async move {
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

      return Task::perform(async move {
        match confirmed_prompt.in_developer_mode {
          false => api::install_plugin(&confirmed_prompt.path).await.map_err(|e| e.to_string()),
          true => api::install_plugin_in_developer_mode(&confirmed_prompt.path).await.map_err(|e| e.to_string()),
        }
      }, Message::InstallResponse);
    },
    Message::InstallResponse(response) => {
      match response {
        Ok(()) => {
          return Task::perform(async {get_plugins().await.map_err(|e| e.to_string())}, Message::InstallGetPlugins);
        },
        Err(e) => {
          warn!("Could not install plugin: {}", e);
          dashboard.dialog = Some(Dialog::Error(format!("Could not install plugin: {}", e).to_string()));
        }
      }
    },
    Message::InstallGetPlugins(response) => {
      dashboard.dialog = None;
      return Task::done(Message::GetPluginsResponse(response));
    },
    Message::CloseDialog => {
      dashboard.dialog = None;
    },
    Message::ToLogs => {
      let (logs_view, logs_message) = logs::Logs::new();

      dashboard.view = View::Logs(logs_view);

      return logs_message.map(Message::Logs);
    },
    Message::PluginList(view::plugin_list::Message::ToPlugin(name)) => {
      let plugin = dashboard.plugins.get(&name);
      match plugin {
        Some(_) => {
          dashboard.view = View::Plugin(view::plugin::Plugin::new(name.clone()));
        },
        None => {
        }
      }
    },
    Message::ToggleSidebar => {
      dashboard.sidebar_minimized.transition(!dashboard.sidebar_minimized.value, Instant::now());
    },
    // Message decision tree based on view state
    message => match &mut dashboard.view {
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
            return Task::done(Message::ToPluginList);
          }
          _ => (),
        },
        _ => (),
      },
      View::PluginList(plugin_list_view) => match message {
        Message::PluginList(plugin_list_message) => return plugin_list_view.update(plugin_list_message).map(Message::PluginList),
        _ => (),
      }
    },
  }

  Task::none()
}