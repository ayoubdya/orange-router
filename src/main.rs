use clearscreen;
use regex::Regex;
use reqwest::{header, Client};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{error::Error, time::Duration};
use tokio::time::sleep;

fn sha256(s: String) -> String {
  Sha256::digest(s.as_bytes())
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect()
}

async fn get_salt_key(client: &Client) -> Result<String, Box<dyn Error>> {
  let text = client
    .get("http://192.168.11.1/?_type=loginData&_tag=login_token")
    .send()
    .await?
    .text()
    .await?;

  let pattern = Regex::new(r"<ajax_response_xml_root>(.*)</ajax_response_xml_root>").unwrap();
  if let Some(first_find) = pattern.captures(&text).and_then(|c| c.get(1)) {
    Ok(first_find.as_str().to_string())
  } else {
    Err("No match found".into())
  }
}

async fn login(client: &Client) -> Result<(), Box<dyn Error>> {
  dotenv::dotenv().ok();
  let password = std::env::var("PASSWORD").unwrap();

  client
    .post("http://192.168.11.1/?_type=loginData&_tag=login_entry")
    .body(format!(
      "action=login&Password={}&Username=user",
      sha256(format!("{}{}", password, get_salt_key(&client).await?))
    ))
    .send()
    .await?;

  client
    .get("http://192.168.11.1/?_type=menuView&_tag=mmTopology&Menu3Location=0")
    .send()
    .await?;

  Ok(())
}

async fn get_data(client: &Client) -> Result<Value, Box<dyn Error>> {
  let data: Value = client
    .get("http://192.168.11.1/?_type=menuData&_tag=topo_lua.lua")
    .send()
    .await?
    .json()
    .await?;
  Ok(data)
}

fn print_data(data: Value) {
  data["ad"]
    .as_object()
    .unwrap()
    .iter()
    .filter(|(k, _)| *k != "MGET_INST_NUM")
    .for_each(|(_, v)| {
      println!(
        "{:14}  {:17}  {:4}  {:4}  {}",
        v["IpAddr"].as_str().unwrap(),
        v["MacAddr"].as_str().unwrap(),
        bps_to_mbps(v["TxRateBps"].as_str().unwrap()),
        bps_to_mbps(v["RxRateBps"].as_str().unwrap()),
        v["HostName"].as_str().unwrap(),
      )
    });
}

fn bps_to_mbps(s: &str) -> String {
  let mbps: f64 = s.parse().unwrap();
  format!("{:.2}", mbps / 8388608.0) // 8388608 = 8 * 1024 * 1024
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let mut headers = header::HeaderMap::new();

  headers.insert(
    header::CONTENT_TYPE,
    header::HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"),
  );

  let client = Client::builder()
    .default_headers(headers)
    .cookie_store(true)
    .build()?;

  login(&client).await?;

  loop {
    let data = get_data(&client).await?;
    clearscreen::clear().unwrap();
    print_data(data);
    sleep(Duration::from_secs(1)).await;
  }
}
