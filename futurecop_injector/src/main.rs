use std::{io, str::FromStr, time::SystemTime};
use fern::colors::{ColoredLevelConfig, Color};
use log::*;
use clap::Parser;
use clap::builder::TypedValueParser as _;
use iced::{window, Application, Settings, Size};

mod gui;
mod config;
mod view;
mod api;
mod injector;
mod log_subscriber;
mod theme;
mod widget;
mod util;


#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    beta: bool,

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
    
    info!("Starting application");

    gui::ModInjector::run(
        Settings {
            window: window::Settings {
                size: Size::new(800.0, 600.0),
                ..window::Settings::default()
            },
            ..Settings::default()
        }
    )
}