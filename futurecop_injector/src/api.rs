use std::{collections::HashMap, path::PathBuf};

use crate::{config::get_config};
use anyhow::anyhow;
use futures::Future;
use log::debug;
use reqwest::Body;
use serde::{de::DeserializeOwned, Deserialize};
use tokio::fs;
use tokio_util::codec::{BytesCodec, FramedRead};

use futurecop_data::plugin::{Plugin, PluginInfo};


pub fn build_url(path: &str) -> String {
  let config = get_config();

  format!("http://{}{}", config.mod_address, path)
}

pub async fn ping_mod() -> Result<String, anyhow::Error> {
  let ping_response = match reqwest::get(build_url("/ping")).await {
    Ok(response) => response,
    Err(e) => return Err(anyhow!("could not ping mod: {}", e.to_string())),
  };

  let response_txt = match ping_response.text().await {
    Ok(text) => text,
    Err(_) => return Err(anyhow!("received malformed text when pinging mod"))
  };

  Ok(response_txt)
}

pub async fn is_mod_running() -> bool {
  match ping_mod().await {
    Ok(response) => response == "Pong",
    Err(_) => false,
  }
}

pub async fn reload_plugin(name: &str) -> Result<(), anyhow::Error> {
  debug!("Reloading plugin: {}", name);

  let mut body = HashMap::new();
  body.insert("name", name);

  match reqwest::Client::new()
    .put(build_url("/plugin/reload"))
    .json(&body)
    .send()
    .await {
      Ok(_) => Ok(()),
      Err(e) => anyhow::bail!("{:?}", e),
  }  
}

pub async fn install_plugin(path: PathBuf) -> Result<(), anyhow::Error> {
  let file = fs::File::open(path.clone()).await.map_err(|e| anyhow!("Could not open file: {}", e.to_string()))?;

  let stream = FramedRead::new(file, BytesCodec::new());
  let body = Body::wrap_stream(stream);

  let response = reqwest::Client::new()
    .post(build_url("/plugin/install"))
    .body(body)
    .send()
    .await
    .map_err(|e| anyhow!("Could not install plugin: {}", e.to_string()))?;

  if !response.status().is_success() {
    let err = match response.text().await {
      Ok(err) => err,
      Err(err) => err.to_string(),
    };

    return Err(anyhow!("Could not install plugin '{}': {}", path.display(), err));
  }

  Ok(())
}

pub async fn get_plugin_info(path: PathBuf) -> Result<PluginInfo, anyhow::Error> {
  let file = fs::File::open(path.clone()).await.map_err(|e| anyhow!("Could not open file: {}", e.to_string()))?;

  let stream = FramedRead::new(file, BytesCodec::new());
  let body = Body::wrap_stream(stream);

  let response = reqwest::Client::new()
    .put(build_url("/plugin/info"))
    .body(body)
    .send()
    .await
    .map_err(|e| anyhow!("Could not get plugin info of: {}", e.to_string()))?;

  if !response.status().is_success() {
    let entire_response = format!("{:?}", response);

    let err = match response.text().await {
      Ok(err) => err,
      Err(err) => err.to_string(),
    };

    let err = if err.len() <= 0 {
      entire_response
    } else {
      err
    };

    return Err(anyhow!("Get plugin info request returned error: {}", err));
  }

  let plugin_info: PluginInfo = match response.json().await {
    Ok(v) => v,
    Err(e) => return Err(anyhow!("Could not serialize response: {:?}", e)),
  };

  Ok(plugin_info)
}

pub async fn uninstall_plugin(name: String) -> Result<(), anyhow::Error> {
  let mut body = HashMap::new();
  body.insert("name", &name);

  let _ = reqwest::Client::new()
    .post(build_url("/plugin/uninstall"))
    .json(&body)
    .send()
    .await
    .map_err(|e| anyhow!("Could not send request to uninstall plugin: {}", e.to_string()))?
    .error_for_status()
    .map_err(|e| anyhow!("Could not uninstall plugin '{}': {}", name, e.to_string()))?;

  Ok(())
}

pub fn handle_response<T>(request: reqwest::Result<T>) -> Result<T, String> {
  match request {
    Err(e) => Err(format!("Failed to send request: {}", e.to_string())),
    Ok(v) => Ok(v),
  }
}

pub async fn parse_json<T>(response: reqwest::Response) -> Result<T, String> where T: DeserializeOwned {
  match response.json::<T>().await {
    Ok(v) => Ok(v),
    Err(e) => Err(format!("Could not parse response: {}", e.to_string())),
  }
}

pub async fn get_plugins() -> Result<HashMap<String, Plugin>, String> {
  let response = handle_response(reqwest::get(build_url("/plugins")).await)?;

  parse_json(response).await
}