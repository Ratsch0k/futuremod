use iced::{alignment::Vertical, border::Radius, widget::{button, column, container, row, text, text_input, toggler, Space}, Border, Length};
use iced_fonts::Bootstrap;

use crate::{config::Config, theme, widget::{icon_button, icon_with_size, Element}};

use super::{Message, Settings};

pub fn settings_overview<'a>(settings: &'a Settings, config: Config) -> Element<'a, Message> {
  column![settings_heading(settings, &config)]
    .push_maybe(settings.error.as_ref().map(|e| error_box(e)))
    .push(settings_content(&settings))
    .padding(16)
    .spacing(16)
    .into()
}

fn error_box<'a>(error: &'a String) -> Element<'a, Message> {
  container(
    column![
      row![
        icon_with_size(Bootstrap::ExclamationTriangle, 20),
        text("Error").size(20),
        Space::with_width(Length::Fill),
        icon_button(Bootstrap::X)
          .on_press(Message::ClearError)
          .class(theme::Button::Text)
          .padding([4.0, 8.0]),
      ]
        .spacing(8)
        .align_y(Vertical::Center),
      text(error),
    ]
      .spacing(4)
  )
    .padding(12)
    .class(theme::Container::Danger)
    .into()
}

fn settings_heading<'a>(settings: &'a Settings, config: &Config) -> Element<'a, Message> {
  let settings_changed = settings != config;

  column![
    row![
      text("Settings").size(24),
      Space::with_width(Length::Fill),
      row![
        button("Reset")
          .on_press_maybe(if settings_changed {Some(Message::Reset)} else {None}),
        button("Save")
          .class(if settings != config {
            theme::Button::Primary
          } else {
            theme::Button::Default
          })
          .on_press_maybe(if settings_changed {
            Some(Message::SaveChanges)
          } else {
            None
          }),
      ]
          .spacing(8.0)
    ]
      .align_y(Vertical::Center),
    text("Configure the settings of FutureMod. For settings to take effect, FutureMod must be restarted."),
  ]
    .into()
}

fn settings_content<'a>(settings: &'a Settings) -> Element<'a, Message> {
  column![
    settings_section(
      "Mod Path",
      "Set the path to the FutureMod engine DLL that is injected into the DLL.",
      row![
        container(text(settings.mod_path.as_str()).wrapping(text::Wrapping::Glyph))
          .padding(8)
          .height(36)
          .width(Length::Fill)
          .class(theme::Container::Custom(Box::new(|theme| container::Style {
            border: Border {
              color: theme.palette.background.medium.color,
              width: 1.0,
              radius: Radius {
                top_left: 12.0,
                top_right: 0.0,
                bottom_left: 12.0,
                bottom_right: 0.0,
              },
            },
            ..theme::container::appearance(theme, &theme::Container::Box)
          }))),
        button("Select")
          .height(36)
          .width(100)
          .on_press(Message::SelectModPath)
          .class(theme::Button::Custom(Box::new(|theme, status| button::Style {
            border: Border {
              radius: Radius {
                top_left: 0.0,
                bottom_left: 0.0,
                top_right: 12.0,
                bottom_right: 12.0,
              },
              ..Border::default()
            },
            ..button::Catalog::style(theme, &theme::Button::Primary, status)
          })))
          ,
      ]
        .align_y(Vertical::Center)
        .width(Length::Fill)
    ),
    settings_section(
      "Mod Address",
      "The IP address and port where the FutureMod engine is listening. By default this should be on localhost with port 8080.",
      text_input("Mod address", &settings.mod_address).on_input(Message::ModAddressChanged),
    ),
    settings_section(
      "Process Name",
      "FutureMod uses this name to identify the FutureCop's process. You should not have to change this value.",
      text_input("Process Name", &settings.process_name).on_input(Message::ProcessNameChanged),
    ),
    settings_section(
      "Requires Admin",
      "If you must run FutureCop with elevated privileges such as an administrator, FutureMod is not able to inject the FutureMod engine into the game. In this case you can set this option to true. Then, FutureMod must be started as admin.",
      toggler(settings.require_admin)
        .label("Requires Admin")
        .on_toggle(Message::RequireAdminChanged),
    )
  ]
    .spacing(24.0)
    .into()
}

fn settings_section<'a>(title: &'a str, description: &'a str, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
  column![
    text(title).size(20),
    Space::with_height(8.0),
    text(description),
    Space::with_height(4.0),
    content.into(),
  ]
    .into()
}