use std::{collections::HashMap, path::PathBuf};

use futuremod_data::plugin::{Plugin, PluginInfo};
use iced::{Command, Subscription};

use crate::{config::get_config, logs, widget::Element, view};

use super::{components, state};


/// Main dashboard.
/// 
/// After successfully injecting FutureMod into the game,
/// the user is presented with this UI.
/// It lists all installed plugins, and offers the user to 
/// install new plugins, access the settings, logs and other
/// functionality.
#[derive(Debug, Clone)]
pub struct Dashboard {
  pub(super) is_developer: bool,
  pub(super) plugins: HashMap<String, Plugin>,
  pub(super) view: Option<View>,
  pub(super) logs: logs::state::Logs,
  pub(super) dialog: Option<Dialog>,
}

#[derive(Debug, Clone)]
pub enum Dialog {
  InstallationPrompt(InstallConfirmationPrompt),
  UninstallPrompt(String),
  Error(String),
}

#[derive(Debug, Clone)]
pub enum View{
  Logs(view::logs::Logs),
  Plugin(view::plugin::Plugin),
}

#[derive(Debug, Clone)]
pub enum Message {
  ToPlugins,
  ToSettings,
  ToLogs,
  Logs(view::logs::Message),
  ToPlugin(String),
  Plugin(view::plugin::Message),
  Enable(String),
  EnableResponse(Result<(), String>),
  Disable(String),
  DisableResponse(Result<(), String>),
  #[allow(unused)]
  Reload(String),
  ReloadResponse(Result<(), String>),
  #[allow(unused)]
  Uninstall(String),
  #[allow(unused)]
  UninstallPrompt(String),
  UninstallResponse(Result<(), String>),
  LogEvent(logs::subscriber::Event),
  GetPluginsResponse(Result<HashMap<String, Plugin>, String>),
  ResetView,
  StartInstallation,
  StartDevelopmentInstallation,
  OpenInstallConfirmationPromptDialog(Result<InstallConfirmationPrompt, String>),
  CloseInstallConfirmationPromptDialog,
  ConfirmInstallation(InstallConfirmationPrompt),
  InstallResponse(Result<(), String>),
  InstallGetPlugins(Result<HashMap<String, Plugin>, String>),
  #[allow(unused)]
  OpenDialog(Dialog),
  CloseDialog,
}

#[derive(Debug, Clone)]
pub struct InstallConfirmationPrompt {
  pub plugin: PluginInfo,
  pub path: PathBuf,
  pub in_developer_mode: bool,
}

impl Dashboard {
  pub fn new(plugins: HashMap<String, Plugin>, is_developer: bool) -> Self {
    Dashboard {
      is_developer,
      plugins,
      view: None,
      logs: logs::state::Logs::default(),
      dialog: None,
    }
  }

  pub fn update(&mut self, message: Message) -> Command<Message> {
    state::update(self, message)
  }

  pub fn view(&self) -> Element<'_, Message> {
    components::dashboard(self)
  }

  pub fn subscription(&self) -> Subscription<Message> {
    let config = get_config();
    
    crate::logs::subscriber::connect(config.mod_address.clone()).map(Message::LogEvent)
  }
}