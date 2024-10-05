use std::{io, str::FromStr, time::SystemTime};
use fern::colors::{ColoredLevelConfig, Color};
use iced_fonts::BOOTSTRAP_FONT_BYTES;
use log::*;
use clap::Parser;
use clap::builder::TypedValueParser as _;
use iced::Size;

mod gui;
mod config;
mod view;
mod api;
mod injector;
mod theme;
mod widget;
mod util;
mod palette;
mod logs;


#[derive(Parser)]
struct Cli {
    #[arg(
        long,
        default_value_t = log::LevelFilter::Info,
        value_parser = clap::builder::PossibleValuesParser::new(
            ["DEBUG", "INFO", "WARN", "ERROR"]
        ).map(|s| log::LevelFilter::from_str(&s).unwrap())
    )]
    log_level: log::LevelFilter,

    #[arg(short, long, default_value_t = String::from("config.json"))]
    config: String,

    #[arg(long, default_value_t = false, help = "Enable developer mode")]
    developer: bool,
}

fn main() -> iced::Result {
    let args = Cli::parse();

    let colors = ColoredLevelConfig::new()
        .info(Color::Cyan)
        .warn(Color::BrightYellow)
        .error(Color::BrightRed);

    match fern::Dispatch::new()
        .level(
            args.log_level,
        )
        .level_for("wgpu_hal", log::LevelFilter::Error)
        .level_for("wgpu_core", log::LevelFilter::Error)
        .level_for("cosmic_text", log::LevelFilter::Error)
        .level_for("iced_wgpu", log::LevelFilter::Error)
        .level_for("naga", LevelFilter::Error)
        .level_for("reqwest", log::LevelFilter::Error)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}] {} - {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .chain(io::stdout())
        .apply()
    {
        Err(e) => println!("Could not configure logging: {}", e),
        _ => (),
    }

    match config::init(&args.config) {
        Ok(_) => (),
        Err(e) => panic!("{}", e)
    }

    if args.developer {
        info!("Starting application in developer mode")
    } else {
        info!("Starting application");
    }

    iced::application::<gui::ModInjector, gui::Message, crate::theme::Theme, iced::Renderer>(gui::title, gui::update, gui::view)
        .subscription(gui::subscription)
        .theme(gui::theme)
        .window_size(Size::new(1024.0, 800.0))
        .font(BOOTSTRAP_FONT_BYTES)
        .antialiasing(true)
        .run_with(move || gui::ModInjector::new(args.developer))
}
