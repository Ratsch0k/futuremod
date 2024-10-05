use std::{collections::HashMap, path::PathBuf, time::Instant};

use futuremod_data::plugin::{Plugin, PluginInfo};
use iced::{window::frames, Subscription, Task};
use lilt::{Animated, Easing};

use crate::{config, logs, view::{self, plugin_list}, widget::Element};

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
  pub(super) view: View,
  pub(super) logs: logs::state::Logs,
  pub(super) dialog: Option<Dialog>,
  pub(super) sidebar_minimized: Animated<bool, Instant>,
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
  PluginList(view::plugin_list::PluginList)
}

#[derive(Debug, Clone)]
pub enum Message {
  ToPluginList,
  PluginList(view::plugin_list::Message),
  #[allow(unused)]
  ToSettings,
  ToLogs,
  Logs(view::logs::Message),
  Plugin(view::plugin::Message),
  EnableResponse(Result<(), String>),
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
  OpenInstallConfirmationPromptDialog(Result<InstallConfirmationPrompt, String>),
  ConfirmInstallation(InstallConfirmationPrompt),
  InstallResponse(Result<(), String>),
  InstallGetPlugins(Result<HashMap<String, Plugin>, String>),
  #[allow(unused)]
  OpenDialog(Dialog),
  CloseDialog,
  ToggleSidebar,
  Tick,
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
      view: View::PluginList(plugin_list::PluginList::new()),
      logs: logs::state::Logs::default(),
      dialog: None,
      sidebar_minimized: Animated::new(false).duration(250.0).easing(Easing::EaseOut),
    }
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    state::update(self, message)
  }

  pub fn view(&self) -> Element<'_, Message> {
    components::dashboard(self)
  }

  pub fn subscription(&self) -> Subscription<Message> {
    let config = config::get();
    
    Subscription::batch([
      Subscription::run_with_id("log_websocket", crate::logs::subscriber::connect(config.mod_address.clone())).map(Message::LogEvent),
      frames().map(|_| Message::Tick),
    ])
  }
}