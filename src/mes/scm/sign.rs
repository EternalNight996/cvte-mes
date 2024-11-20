#![allow(unused)]
use e_utils::chrono::Utc;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use serde_json::json;
use std::collections::BTreeMap;
#[derive(Debug, Clone)]
pub struct DefaultSignSDK {}
impl DefaultSignSDK {
  // 常量定义
  const API_KEY: &'static str = "apikey";
  const TIMESTAMP: &'static str = "timestamp";
  const SIGN: &'static str = "sign";
  const API_SECRET: &'static str = "apiSecret";

  // 测试凭证
  const API_KEY_TEST: &'static str = "8yJ7VsohlsnY56BZNUOoqTgyfmg6Yuci";
  const API_KEY_SECRET: &'static str = "MOKA";
}

impl DefaultSignSDK {
  /// 获取签名
  pub fn get_sign(param_map: &BTreeMap<String, String>) -> String {
    Self::get_md5_str(&Self::get_plain_text(param_map))
  }

  /// 获取明码文本
  fn get_plain_text(param_map: &BTreeMap<String, String>) -> String {
    let api_key = param_map.get(Self::API_KEY).cloned().unwrap_or_default();
    let timestamp = param_map.get(Self::TIMESTAMP).cloned().unwrap_or_default();
    let api_secret = param_map.get(Self::API_SECRET).cloned().unwrap_or_default();

    let start = format!("{}={}", Self::TIMESTAMP, timestamp);
    let end = format!("&{}={}&{}={}", Self::API_SECRET, api_secret, Self::API_KEY, api_key);

    let content: String = param_map
      .iter()
      .filter(|(key, _)| ![Self::API_KEY, Self::TIMESTAMP, Self::API_SECRET].contains(&key.as_str()))
      .map(|(key, value)| format!("&{}={}", key, value))
      .collect();

    if content.is_empty() {
      format!("{}{}", start, end)
    } else {
      format!("{}{}{}", start, content, end)
    }
  }

  /// MD5加密
  fn get_md5_str(plain_text: &str) -> String {
    let digest = md5::compute(plain_text);
    format!("{:x}", digest)
  }

  /// 示例请求
  pub async fn example_request() -> e_utils::AnyResult<String> {
    let mut param_map = BTreeMap::new();
    param_map.insert("lineLotNumber".to_string(), "MOKA-SKZBU23110001".to_string());

    // 获取时间戳
    let timestamp = (Utc::now().timestamp_millis() + 6000000).to_string();
    param_map.insert(Self::API_KEY.to_string(), Self::API_KEY_TEST.to_string());
    param_map.insert(Self::API_SECRET.to_string(), Self::API_KEY_SECRET.to_string());
    param_map.insert(Self::TIMESTAMP.to_string(), timestamp.clone());

    // 获取签名
    let sign = Self::get_sign(&param_map);
    println!("时间戳：{}", timestamp);
    println!("签名：{}", sign);

    // 构建请求头
    let headers = HeaderMap::from_iter([
      (CONTENT_TYPE, HeaderValue::from_static("application/json;charset=UTF-8")),
      (HeaderName::from_static("apikey"), HeaderValue::from_str(Self::API_KEY_TEST)?),
      (HeaderName::from_static("timestamp"), HeaderValue::from_str(&timestamp)?),
      (HeaderName::from_static("sign"), HeaderValue::from_str(&sign)?),
    ]);
    // 发送请求
    let client = reqwest::Client::new();
    let response = client
      .post("https://itgw-uat.cvte.com/env-101/scm/scmwip/gw-wip/wip/api/mo/factory")
      .headers(headers)
      .json(&json!(param_map))
      .send()
      .await?
      .text()
      .await?;

    println!("{}", response);
    Ok(response)
  }
}

#[tokio::main]
async fn main() -> e_utils::AnyResult<()> {
  DefaultSignSDK::example_request().await?;
  Ok(())
}
