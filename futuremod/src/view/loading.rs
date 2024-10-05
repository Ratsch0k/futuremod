use std::{collections::HashMap, path::{Path, PathBuf}, time::{Duration, SystemTime}};
use futuremod_data::plugin::Plugin;
use iced::{widget::{column, container, row, text, Column}, Alignment, Length, Padding, Task};
use iced_fonts::Bootstrap;
use log::*;
use rfd::FileDialog;

use crate::{api::{self, is_mod_running}, config, injector::{get_future_cop_handle, inject_mod}, theme, widget::{button, icon_with_size, Element}};

const MAX_INJECTION_TRIES: u8 = 3;
const INJECTION_WAIT_TIMEOUT_SECONDS: u64 = 5;



#[derive(Debug)]
pub enum Loading {
  NoPath,
  WaitingForProgram{mod_path: PathBuf, error: Option<String>},
  InjectionError{mod_path: PathBuf, error: String},
  /// State while waiting for the injected mod to start.
  /// 
  /// For some reason, injection isn't always successful on the first try.
  /// Therefore, we inject the mod again after the mod server didn't start for
  /// some time. If injection tries exceed a threshold, we show an error.
  /// This variant keeps track of the time when the mod was injected in this injection
  /// attempt and how many attempts were already made.
  WaitingForMod{since: SystemTime, injection_attempts: u8, mod_path: PathBuf},
  FetchingPlugins,
  FetchingPluginError(String),
}

#[derive(Debug, Clone)]
pub enum Message {
  OpenPathSelection,
  ChangeModPath(PathBuf),
  CheckIfStarted,
  IsModActive(bool),
  PluginResponse(Result<HashMap<String, Plugin>, String>),
  GotPlugins(HashMap<String, Plugin>)
}

impl Loading {
  pub fn new() -> (Self, Task<Message>) {
    let config = config::get();

    let mod_path = Path::new(&config.mod_path).to_path_buf();

    match mod_path.exists() {
      true => {
        info!("found mod file, checking if mode is active");
        (Loading::WaitingForProgram{mod_path, error: None}, Task::perform(async {}, |_| Message::CheckIfStarted))
      }
      false => {
        info!("didn't found mod file, requesting user to select one");
        (Loading::NoPath, Task::none())
      }
    }
  }

  pub fn view(&self) -> Element<Message> {
    let content: Column<Message, theme::Theme> = match self {
      Loading::WaitingForProgram{mod_path, error } => {
        let error_message = error.as_ref().map(|e| 
          container(
            container(
              column![
                row![icon_with_size(Bootstrap::ExclamationTriangle, 20), text("Error").size(20)].spacing(8),
                text(e),
              ]
                .spacing(8)
            )
              .class(theme::Container::Danger)
              .padding(16)
          )
            .padding(Padding::default().top(32))
        );

        column![
          text("Waiting for program to start")
            .size(24),
          container(
            text(mod_path.to_str().unwrap_or("error parsing mod path"))
          ).padding(Padding{top: 0.0, left: 0.0, bottom: 8.0, right: 0.0}),
          button("Change Mod")
            .on_press(Message::OpenPathSelection),
        ]
          .push_maybe(error_message)
          .into()
      },
      Loading::WaitingForMod{..} => {
        column![text("Waiting for mod to start...")].into()
      },
      Loading::InjectionError{error, ..} => {
        column![
          text(error),
          button("Retry").on_press(Message::CheckIfStarted),
        ].into()
      }
      Loading::NoPath => {
        column![
          text("Mod Not Found")
            .size(24),
          text("Could not find mod, please select the mod"),
          button("SELECT")
            .on_press(Message::OpenPathSelection),
        ].into()
      },
      Loading::FetchingPlugins => {
        column![text("Fetching plugins")].into()
      },
      Loading::FetchingPluginError(e) => {
        column![text(format!("Error while fetching plugins: {}", e))].into()
      }
    };

    return container(
      row![
        content
          .spacing(4)
          .align_x(Alignment::Center)
          .width(Length::Fill)
      ]
      .height(Length::Fill)
      .align_y(Alignment::Center)
    ).into();
}

  pub fn update(&mut self, msg: Message) -> Task<Message> {
    match self {
      Loading::WaitingForProgram { mod_path, error } => match msg {
        Message::CheckIfStarted => {
          info!("Check if FutureCop has started");
          let mod_path = mod_path.clone();

          return self.try_to_inject_mod(mod_path);
        },
        Message::OpenPathSelection => return self.pick_mod_path(),
        Message::ChangeModPath(path) => {
          let path_str = match path.to_str(){
            Some(v) => v.to_string(),
            None => {
              *error = Some(String::from("Invalid path. Please select another file."));
              return Task::none();
            }
          };

          if let Err(e) = config::update(move |config| {
            config.mod_path = path_str.clone();
          }) {
            *error = Some(e.to_string());
            return Task::none();
          }

          *error = None;

          *self = Loading::WaitingForProgram { mod_path: path, error: None };
        },
        _ => (),
      },
      Loading::InjectionError{mod_path, ..} => match msg {
        Message::CheckIfStarted => {
          info!("Retry injecting mod");
          let mod_path = mod_path.clone();
          return self.try_to_inject_mod(mod_path);
        },
        _ => (),
      },
      Loading::WaitingForMod{since, injection_attempts: injection_tries, mod_path} => match msg {
        Message::IsModActive(is_active) => match is_active {
          true => {
            *self = Loading::FetchingPlugins;

            return Task::perform(async {api::get_plugins().await.map_err(|e| e.to_string())}, Message::PluginResponse);
          },
          false => {
            // Check how much time has passed since waiting for the mod
            let now = SystemTime::now();

            // If we waited to long for the mod to start, something went wrong. Either show an error or inject againt
            if now.duration_since(*since).unwrap() > Duration::from_secs(INJECTION_WAIT_TIMEOUT_SECONDS) {
              // If we already tried injecting a max amount of time, show the user an error
              if *injection_tries >= MAX_INJECTION_TRIES {
                warn!("Was never able to successfully inject the mod. Showing error");
                *self = Loading::InjectionError { mod_path: mod_path.clone().to_path_buf(), error: String::from("Was not able to inject the mod") };
                return Task::none();
              }
            // If there are still some injection tries left and a timeout occurred, try injecting the mod again.
              info!("Already waiting for the mod for over 5 seconds. Something went wrong. Retrying to inject mod.");
              let mod_path = mod_path.clone().to_path_buf();
              *self = Loading::WaitingForMod { since: SystemTime::now(), injection_attempts: *injection_tries + 1, mod_path: mod_path.clone() };
              return self.try_to_inject_mod(mod_path.clone());
            }

            // Check if the mod is active
            info!("Checking if mod is active");

            return Task::perform(
              async {
                tokio::time::sleep(Duration::from_millis(500)).await;

                api::is_mod_running().await
              },
              Message::IsModActive,
            );
          }
        }
        _ => (),
      },
      Loading::NoPath => match msg {
        Message::OpenPathSelection => return self.pick_mod_path(),
        _ => (),
      },
      Loading::FetchingPlugins => match msg {
        Message::PluginResponse(response) => match response {
          Ok(plugins) => {
            return Task::perform(async {plugins}, Message::GotPlugins);
          },
          Err(e) => {
            *self = Loading::FetchingPluginError(e);
          }
        },
        _ => (),
      },
      _ => (),
    }

    Task::none()
  }

  fn pick_mod_path(&mut self) -> Task<Message> {
    info!("Prompting user to pick the mod file");
    match FileDialog::new().set_directory(".").pick_file() {
      Some(path) => {
        info!("Changing mod path to: {}", path.to_str().unwrap());

        Task::done(Message::ChangeModPath(path))
      },
      None => Task::none()
    }
  }

  fn try_to_inject_mod(&mut self, mod_path: PathBuf) -> Task<Message> {
    info!("Trying to inject mod");
    let config = config::get();

    info!("Getting handle to FutureCop process");
    match get_future_cop_handle(config.require_admin) {
      Ok(optional_handle) => match optional_handle {
        Some(handle) => {
          info!("Got handle to FutureCop process");
          match inject_mod(handle, mod_path.to_str().unwrap().to_string()) {
            Err(e) => {
              warn!("Error while injecting the mod into FutureCop: {}", e);
              *self = Loading::InjectionError{
                error: format!("Could not inject the mod: {}", e).to_string(),
                mod_path,
              };
              return Task::none();
            },
            Ok(_) => {
              info!("Successfully injected mod");
              *self = Loading::WaitingForMod{since: SystemTime::now(), injection_attempts: 0, mod_path};
              return check_if_mod_running();
            }
          }
        },
        None => {
          info!("Process not started yet");
        },
      },
      Err(e) => {
        warn!("Error while trying to the a handle to the FutureCop process: {}", e);
      }
    }

    info!("Injection not successful, trying again in 100ms");
    return Task::perform(async {tokio::time::sleep(Duration::from_millis(100))}, |_| Message::CheckIfStarted);
  }
}

fn check_if_mod_running() -> Task<Message> {
  Task::perform(is_mod_running(), Message::IsModActive)
}
