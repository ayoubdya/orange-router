use clearscreen;
use regex::Regex;
use reqwest::{header, Client, StatusCode};
use serde_json::Value;
use std::{error::Error, time::Duration};
use tokio::time::sleep;

mod utils;

struct RouterScraper {
  client: Client,
  password: String,
}

impl RouterScraper {
  fn new() -> Result<Self, Box<dyn Error>> {
    dotenv::dotenv().ok();
    let password = std::env::var("PASSWORD").expect("PASSWORD env var not set");

    let mut headers = header::HeaderMap::new();
    headers.insert(
      header::CONTENT_TYPE,
      header::HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"),
    );

    let client = Client::builder()
      .default_headers(headers)
      .cookie_store(true)
      .build()?;

    Ok(Self { client, password })
  }

  async fn get_salt_key(&self) -> Result<String, Box<dyn Error>> {
    let text = self
      .client
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

  async fn login(&self) -> Result<(), Box<dyn Error>> {
    let salt_key = self.get_salt_key().await?;
    let encrypted_password = utils::sha256(format!("{}{}", self.password, salt_key));

    self
      .client
      .post("http://192.168.11.1/?_type=loginData&_tag=login_entry")
      .body(format!(
        "action=login&Password={}&Username=user",
        encrypted_password
      ))
      .send()
      .await?;

    match self
      .client
      .get("http://192.168.11.1/?_type=menuView&_tag=mmTopology&Menu3Location=0")
      .send()
      .await?
      .status()
    {
      StatusCode::OK => Ok(()),
      _ => Err("Login failed".into()),
    }
  }

  async fn logout(&self) -> Result<(), Box<dyn Error>> {
    self
      .client
      .post("https://192.168.11.1/?_type=loginData&_tag=logout_entry")
      .body("_type=loginData&_tag=logout_entry")
      .send()
      .await?;
    Ok(())
  }

  async fn get_data(&self) -> Result<Value, Box<dyn Error>> {
    let data: Value = self
      .client
      .get("http://192.168.11.1/?_type=menuData&_tag=topo_lua.lua")
      .send()
      .await?
      .json()
      .await?;
    Ok(data)
  }
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
        utils::bps_to_mbps(v["TxRateBps"].as_str().unwrap()),
        utils::bps_to_mbps(v["RxRateBps"].as_str().unwrap()),
        v["HostName"].as_str().unwrap(),
      )
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let scraper = RouterScraper::new()?;
  scraper.login().await?;

  loop {
    let data = scraper.get_data().await?;
    clearscreen::clear().unwrap();
    print_data(data);
    sleep(Duration::from_secs(1)).await;
  }
}
