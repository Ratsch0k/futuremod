use std::{collections::HashMap, path::{Path, PathBuf}, sync::{Arc, RwLock}, thread::JoinHandle, time::SystemTime};
use anyhow::{Error, anyhow};
use axum::{
    body::Bytes, extract::{ws::{Message, WebSocket, WebSocketUpgrade}, BodyStream}, http::StatusCode, response::{IntoResponse, Response}, routing::{get, post, put}, BoxError, Json, Router,
};
use futurecop_data::plugin::PluginInfo;
use kv::Key;
use log::*;
use serde::{Serialize, Deserialize};
use tokio::{fs, io, runtime::Runtime, sync::broadcast::{self, Receiver, Sender}};
use std::thread;
use futures::Stream;
use rand::distributions::{Alphanumeric, DistString};
use futures::TryStreamExt;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;

use crate::{config::Config, plugins::{plugin_info::{load_plugin_info, PluginInfoError}, plugin_manager::{GlobalPluginManager, PluginInstallError}}};

use super::plugins::{PluginManager, plugin_manager::PluginManagerError};

lazy_static! {
    pub static ref LOG_PUBLISHER: LogPublisher = LogPublisher::new();
    static ref LOG_HISTORY: Arc<RwLock<Vec<(u64, LogRecord)>>> =  Arc::new(RwLock::new(Vec::new()));
}

/// Start the mod server in a separate thread.
/// 
/// Returns the thread's handle.
pub fn start_server(config: Config) -> JoinHandle<()> {
    let handle = thread::spawn(move || {
        let _ = serve(config);
    });

    handle
}

/// Start the server
fn serve(config: Config) -> Result<(), Error> {
    let result = std::panic::catch_unwind(|| {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let app = Router::new()
                .route("/ping", get(ping))
                .route("/read", post(read_memory))
                .route("/read-hex", post(read_memory_hex))
                .route("/plugins", get(get_plugins))
                .route("/plugin/enable", put(enable_plugin))
                .route("/plugin/disable", put(disable_plugin))
                .route("/plugin/reload", put(reload_plugin))
                .route("/plugin/install", post(install_plugin))
                .route("/plugin/uninstall", post(uninstall_plugin))
                .route("/plugin/info", put(get_plugin_info))
                .route("/log", get(log_handler));

            axum::Server::bind(&format!("{}:{}", config.server.host, config.server.port).parse().unwrap())
                .serve(app.into_make_service())
                .await
                .unwrap();
        });
    });

    match result {
        Err(_) => Err(anyhow!("The server panicked")),
        _ => Ok(())
    }
}

async fn log_handler(
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    debug!("Registering new log consumer");
    ws.on_upgrade(handle_log)
}

async fn handle_log(mut socket: WebSocket) {
    let mut log_receiver = LOG_PUBLISHER.subscribe();

    let (last_history_id, log_history) = {
        let log_history = LOG_HISTORY.read().unwrap();
        let mut copy_of_log_history: Vec<(u64, LogRecord)> = Vec::new();
        let last_seen_id_of_history = log_history.len() as u64;

        for (record_id, log_record) in log_history.iter() {
            copy_of_log_history.push((*record_id, log_record.clone()));
        }
        
        (last_seen_id_of_history, copy_of_log_history)
    };
    

    for record in log_history.iter() {
        let log_json_message = match serde_json::to_string(&record.1) {
            Ok(m) => m,
            Err(_) => continue,
        };

        match socket.send(Message::Text(log_json_message)).await {
            Ok(_) => (),
            Err(e) => {
                warn!("Could not send log record: {}", e);
                return;
            },
        }
    }


    while let Ok((id, message)) = log_receiver.recv().await {
        let message = match serde_json::to_string(&message) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if id > last_history_id {
            match socket.send(Message::Text(message)).await {
                Err(_) => return,
                _ => (),
            }
        }
    }
}

async fn ping() -> &'static str {
    "Pong"
}

#[derive(Deserialize)]
struct ReadMemory {
    address: u32,
    size: u32,
}

#[derive(Deserialize)]
struct ReadMemoryHex {
    address: String,
    size: u32,
}

#[derive(Serialize)]
struct Memory {
    value: Vec<u8>,
}

async fn read_memory(Json(payload): Json<ReadMemory>) -> (StatusCode, Json<Memory>) {
    let memory;

    unsafe {
        let mut raw_bytes: Vec<u8> = Vec::new();
        let raw_address = payload.address as *const u8;

        for i in 0..payload.size {
            raw_bytes.push(*(raw_address.offset(i as isize)));
        }

        memory = Memory {
            value: raw_bytes,
        }
    }

    (StatusCode::OK, Json(memory))
}

#[derive(Debug)]
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}


impl<E> From<E> for AppError where E: Into<anyhow::Error> {
    fn from(value: E) -> Self {
        AppError(value.into())
    }
}



async fn read_memory_hex(Json(payload): Json<ReadMemoryHex>) -> impl IntoResponse {
    let memory;
    let address = match i64::from_str_radix(payload.address.as_str(), 16) {
        Ok(a) => a,
        Err(err) => return Err(AppError(anyhow!("could not parse address: {}", err))),
    };

    unsafe {
        let mut raw_bytes: Vec<u8> = Vec::new();
        let raw_address = address as *const u8;

        for i in 0..payload.size {
            raw_bytes.push(*(raw_address.offset(i as isize)));
        }

        memory = Memory {
            value: raw_bytes,
        }
    }

    Ok(Json(memory))
}

fn with_plugin_manager_mut<F, R>(f: F) -> Result<R, AppError>
where F: Fn(&mut PluginManager) -> R {
    match GlobalPluginManager::get().lock() {
        Ok(mut plugin_manager) => {
            Ok(f(&mut plugin_manager))
        },
        Err(e) => Err(AppError(anyhow!("could not get lock to plugin manager: {:?}", e))),
    }
}

fn with_plugin_manager<F, R>(f: F) -> Result<R, anyhow::Error> where F: Fn(&PluginManager) -> Result<R, anyhow::Error> {
    match GlobalPluginManager::get().lock() {
        Ok(mut plugin_manager) => {
            Ok(f(&mut plugin_manager)?)
        },
        Err(e) => Err(anyhow!("Could not get lock to plugin manager: {:?}", e)),
    }
}

async fn get_plugins() -> Result<Json<HashMap<String, futurecop_data::plugin::Plugin>>, String> {
    GlobalPluginManager::with_plugin_manager(|plugin_manager| {
        let plugins = plugin_manager.get_plugins();

        let mut plugin_response: HashMap<String, futurecop_data::plugin::Plugin> = HashMap::new();

        for (name, plugin) in plugins.iter() {
            plugin_response.insert(name.clone(), plugin.clone().into());
        }

        Ok(Json(plugin_response))
    }).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct PluginByName {
    name: String,
}

async fn enable_plugin(Json(payload): Json<PluginByName>) -> impl IntoResponse {
    with_plugin_manager_mut(|plugin_manager| -> Response {
        match plugin_manager.enable_plugin(&payload.name) {
            Err(e) => match e {
                PluginManagerError::PluginNotFound => {
                    (StatusCode::NOT_FOUND, AppError(anyhow!("plugin doesn't exist"))).into_response()
                },
                e => {
                    (StatusCode::INTERNAL_SERVER_ERROR, AppError(anyhow!("could not enable plugin: {:?}", e))).into_response()
                }
            },
            _ => StatusCode::NO_CONTENT.into_response(),
        }
    })
}

async fn disable_plugin(Json(payload): Json<PluginByName>) -> impl IntoResponse {
    with_plugin_manager_mut(|plugin_manager| -> Response {
        match plugin_manager.disable_plugin(&payload.name) {
            Err(e) => match e {
                PluginManagerError::PluginNotFound => {
                    (StatusCode::NOT_FOUND, AppError(anyhow!("plugin doesn't exist"))).into_response()
                },
                e => {
                    (StatusCode::INTERNAL_SERVER_ERROR, AppError(anyhow!("could not enable plugin: {:?}", e))).into_response()
                }
            },
            _ => StatusCode::NO_CONTENT.into_response(),
        }
    })
}

async fn reload_plugin(Json(payload): Json<PluginByName>) -> impl IntoResponse {
    with_plugin_manager_mut(|plugin_manager| -> Response {
        match plugin_manager.reload_plugin(&payload.name) {
            Err(e) => match e {
                PluginManagerError::PluginNotFound => {
                    (StatusCode::NOT_FOUND, AppError(anyhow!("plugin doesn't exist"))).into_response()
                },
                e => (StatusCode::INTERNAL_SERVER_ERROR, AppError(anyhow!("could not reload plugin: {:?}", e))).into_response(),
            }
            _ => StatusCode::NO_CONTENT.into_response(),
        }
    })
}

const TEMPORARY_DIRECTORY: &str = "fcop";

enum InstallError {
    ExtractionError(String),
    Other(String),
}


async fn get_plugin_info(request: BodyStream) -> (StatusCode, Result<Json<PluginInfo>, String>) {
    info!("Get plugin info");

    let random_file_name: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    let mut random_file_path = PathBuf::from(random_file_name);
    random_file_path.set_extension("zip");

    let fcop_temp_folder = Path::new(&std::env::temp_dir()).join(PathBuf::from(TEMPORARY_DIRECTORY));
    if !fcop_temp_folder.exists() {
        if let Err(err) = fs::create_dir(&fcop_temp_folder).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, Err(format!("Could not create temporary directory for fcop mod: {}", err.to_string())));
        }
    }

    let temporary_file_path = fcop_temp_folder.join(&random_file_path);
    debug!("Storing incoming plugin package in temporary file: {}", temporary_file_path.to_str().unwrap_or("unknown"));

    match write_to_temp_file(&temporary_file_path, request).await {
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Err(e.0.to_string())),
        _ => (),
    };
    debug!("Wrote plugin file into temporary file");

    info!("Extracting plugin package");
    let temporary_plugin_folder = match extract_temp_file(&temporary_file_path).await {
        Err(e) => match e {
            InstallError::ExtractionError(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, Err(format!("Error while extracting the plugin package: {}", msg))),
            InstallError::Other(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, Err(msg)),
        },
        Ok(v) => v,
    };

    info!("Reading plugin information");
    let info = match load_plugin_info(temporary_plugin_folder.clone()) {
        Err(err) => match err {
            PluginInfoError::FileNotFound => return (StatusCode::BAD_REQUEST, Err("Plugin package doesn't contain a info file".to_string())),
            PluginInfoError::Format(msg) => return (StatusCode::BAD_REQUEST, Err(format!("Plugin info file has invalid format: {}", msg))),
            PluginInfoError::Other(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, Err(format!("Unexpected error while reading the plugin's info file: {}", msg))),
        },
        Ok(v) => v,
    };

    info!("Deleting temporary plugin");
    match fs::remove_dir_all(temporary_plugin_folder).await {
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Err(format!("Error while deleting the temporarily created plugin: {:?}", e))),
        Ok(()) => (),
    };

    (StatusCode::OK, Ok(Json(info)))
}


async fn install_plugin(request: BodyStream) -> (StatusCode, Result<(), String>) {
    info!("Installing new plugin");

    let random_file_name: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    let mut random_file_path = PathBuf::from(random_file_name);
    random_file_path.set_extension("zip");

    let fcop_temp_folder = Path::new(&std::env::temp_dir()).join(PathBuf::from(TEMPORARY_DIRECTORY));
    if !fcop_temp_folder.exists() {
        if let Err(err) = fs::create_dir(&fcop_temp_folder).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, Err(format!("Could not create temporary directory for fcop mod: {}", err.to_string())));
        }
    }

    let temporary_file_path = fcop_temp_folder.join(&random_file_path);
    debug!("Storing incoming plugin package in temporary file: {}", temporary_file_path.to_str().unwrap_or("unknown"));

    match write_to_temp_file(&temporary_file_path, request).await {
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Err(format!("{:?}", e))),
        _ => (),
    };
    debug!("Wrote plugin file into temporary file");

    info!("Extracting plugin package");
    let temporary_plugin_folder = match extract_temp_file(&temporary_file_path).await {
        Err(e) => match e {
            InstallError::ExtractionError(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, Err(format!("Error while extracting the plugin package: {}", msg))),
            InstallError::Other(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, Err(msg)),
        },
        Ok(v) => v,
    };

    info!("Reading plugin information");
    let info = match load_plugin_info(temporary_plugin_folder.clone()) {
        Err(err) => match err {
            PluginInfoError::FileNotFound => return (StatusCode::BAD_REQUEST, Err("Plugin package doesn't contain a info file".to_string())),
            PluginInfoError::Format(msg) => return (StatusCode::BAD_REQUEST, Err(format!("Plugin info file has invalid format: {}", msg))),
            PluginInfoError::Other(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, Err(format!("Unexpected error while reading the plugin's info file: {}", msg))),
        },
        Ok(v) => v,
    };

    let plugin_name = info.name;
    info!("Installing plugin '{}'", plugin_name);

    match with_plugin_manager_mut(move |plugin_manager| {
        plugin_manager.install_plugin_from_folder(&temporary_plugin_folder)
    }) {
        Ok(result) => match result {
            Ok(()) => (StatusCode::OK, Ok(())),
            Err(err) => match err {
                PluginInstallError::AlreadyInstalled => (StatusCode::BAD_REQUEST, Err("plugin is already installed".to_string())),
                PluginInstallError::InvalidName => (StatusCode::BAD_REQUEST, Err("plugin has an invalid name".to_string())),
                PluginInstallError::InfoFile(e) => (StatusCode::BAD_REQUEST, Err(format!("plugin package info error: {:?}", e))),
                PluginInstallError::Plugin(e) => (StatusCode::BAD_REQUEST, Err(format!("Plugin was installed but immediately errored: {:?}", e))),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, Err(format!("Error while installing plugin: {:?}", err))),
            }
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Err(format!("Error while installing plugin: {:?}", err))),
    }
}

async fn write_to_temp_file<S, E>(path_name: &PathBuf, stream: S) -> Result<(), AppError>
where S: Stream<Item = Result<Bytes, E>>, E: Into<BoxError> {
    async {
        debug!("Start extracting to {:?}", path_name);
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        debug!("Create buffered writer");
        let mut file = BufWriter::new(File::create(path_name).await?);

        debug!("Copying the stream into the file");
        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|e| AppError(anyhow!("{}", e.to_string())))
}

async fn extract_temp_file(path: &PathBuf) -> Result<PathBuf, InstallError> {
    // Open the plugin package
    let plugin_package = fs::File::open(path)
        .await
        .map_err(|err| InstallError::Other(err.to_string()))?
        .into_std().await;

    let mut archive = zip::ZipArchive::new(plugin_package).map_err(|err| InstallError::ExtractionError(err.to_string()))?;

    let mut destination = path.clone();
    destination.set_extension("");

    // Actually extract the archive to the destination folder
    archive.extract(&destination).map_err(|err| InstallError::ExtractionError(err.to_string()))?;

    Ok(destination)
}

async fn uninstall_plugin(Json(payload): Json<PluginByName>) -> impl IntoResponse {
    with_plugin_manager_mut(|plugin_manager| {
        match plugin_manager.uninstall_plugin(payload.name.as_str()) {
            Err(e) => match e {
                PluginManagerError::PluginNotFound => return (StatusCode::NOT_FOUND, "plugin not found").into_response(),
                _ => return (StatusCode::INTERNAL_SERVER_ERROR, format!("unexpected error: {:?}", e )).into_response(),
            },
            Ok(_) => StatusCode::NO_CONTENT.into_response(),
        }
    })
}

#[derive(Debug)]
pub struct LogPublisher {
    publisher: Sender<(u64, LogRecord)>,
    _base_rx: Receiver<(u64, LogRecord)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogRecord {
    message: String,
    target: String,
    level: String,
    timestamp: String,
    plugin: Option<String>,
}

impl<'a> From<&log::Record<'a>> for LogRecord {
    fn from(value: &log::Record) -> Self {
        LogRecord {
            message: format!("{}", value.args()),
            target: value.target().to_string(),
            level: value.level().as_str().to_string(),
            timestamp: humantime::format_rfc3339_millis(SystemTime::now()).to_string(),
            plugin: value.key_values().get(Key::from("plugin")).map(|value| value.to_string()),
        }
    }
}

impl LogPublisher {
    fn new() -> Self {
        let (tx, rx) = broadcast::channel::<(u64, LogRecord)>(16);

        LogPublisher {
            publisher: tx,
            _base_rx: rx
        }
    }

    fn subscribe(&self) -> Receiver<(u64, LogRecord)> {
        self.publisher.subscribe()
    }
}

impl Log for LogPublisher {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let mut log_history = LOG_HISTORY.write().unwrap();
        let record_id = log_history.len() as u64;

        let message = (record_id, LogRecord::from(record));

        log_history.push(message.clone());

        let _ = self.publisher.send(message.clone());
    }

    fn flush(&self) {
        
    }
}
