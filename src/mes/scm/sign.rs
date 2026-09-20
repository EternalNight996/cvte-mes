#![allow(unused)]
use e_utils::chrono::Utc;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use serde_json::json;
use std::collections::BTreeMap;

/// 网关鉴权头 key（文档约定）
pub const API_KEY: &str = "apikey";
pub const TIMESTAMP: &str = "timestamp";
pub const SIGN: &str = "sign";
pub const API_SECRET: &str = "apiSecret";

/// MD5 加密为 32 位小写字符串
pub fn get_md5_str(plain_text: &str) -> String {
  let digest = md5::compute(plain_text.as_bytes());
  format!("{:x}", digest)
}

/// 明码串 = timestamp=... + &业务串(字典升序) + &apiSecret=...&apikey=...
/// 文档 §1.4：业务串为字典升序，key 和 value 用 = 拼接，多个 key=value 用 & 拼接
pub fn get_plain_text(
  apikey: &str,
  api_secret: &str,
  timestamp: &str,
  params: &BTreeMap<String, String>,
) -> String {
  let start = format!("{}={}", TIMESTAMP, timestamp);
  let end = format!("&{}={}&{}={}", API_SECRET, api_secret, API_KEY, apikey);
  let content: String = params
    .iter()
    .filter(|(key, _)| ![API_KEY, TIMESTAMP, API_SECRET].contains(&key.as_str()))
    .map(|(key, value)| format!("&{}={}", key, value))
    .collect();
  if content.is_empty() {
    format!("{}{}", start, end)
  } else {
    format!("{}{}{}", start, content, end)
  }
}

/// 计算签名（params 需已包含 timestamp；业务参数按 key 字典升序）
pub fn get_sign(
  apikey: &str,
  api_secret: &str,
  params: &BTreeMap<String, String>,
) -> String {
  let timestamp = params.get(TIMESTAMP).cloned().unwrap_or_default();
  get_md5_str(&get_plain_text(apikey, api_secret, &timestamp, params))
}

/// 组装鉴权请求头：Content-Type / apikey / timestamp / sign
pub fn build_headers(
  apikey: &str,
  api_secret: &str,
  params: &BTreeMap<String, String>,
) -> e_utils::AnyResult<HeaderMap> {
  let timestamp = params.get(TIMESTAMP).cloned().unwrap_or_default();
  let sign = get_sign(apikey, api_secret, params);
  let mut headers = HeaderMap::new();
  headers.insert(
    CONTENT_TYPE,
    HeaderValue::from_static("application/json;charset=UTF-8"),
  );
  headers.insert(
    HeaderName::from_static(API_KEY),
    HeaderValue::from_str(apikey)?,
  );
  headers.insert(
    HeaderName::from_static(TIMESTAMP),
    HeaderValue::from_str(&timestamp)?,
  );
  headers.insert(HeaderName::from_static(SIGN), HeaderValue::from_str(&sign)?);
  Ok(headers)
}

#[derive(Debug, Clone)]
pub struct DefaultSignSDK {}
impl DefaultSignSDK {
  // 测试凭证
  pub const API_KEY_TEST: &'static str = "8yJ7VsohlsnY56BZNUOoqTgyfmg6Yuci";
  pub const API_KEY_SECRET: &'static str = "MOKA";
}

impl DefaultSignSDK {
  /// 获取签名
  pub fn get_sign(param_map: &BTreeMap<String, String>) -> String {
    let api_key = param_map.get(API_KEY).cloned().unwrap_or_default();
    let api_secret = param_map.get(API_SECRET).cloned().unwrap_or_default();
    get_sign(&api_key, &api_secret, param_map)
  }

  /// 示例请求
  pub async fn example_request() -> e_utils::AnyResult<String> {
    let mut param_map = BTreeMap::new();
    param_map.insert("lineLotNumber".to_string(), "MOKA-SKZBU23110001".to_string());

    // 获取时间戳
    let timestamp = (Utc::now().timestamp_millis() + 6000000).to_string();
    param_map.insert(API_KEY.to_string(), Self::API_KEY_TEST.to_string());
    param_map.insert(API_SECRET.to_string(), Self::API_KEY_SECRET.to_string());
    param_map.insert(TIMESTAMP.to_string(), timestamp.clone());

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
    let client = crate::mes::cmes::client();
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sign_matches_document_example() {
    // 文档 §1.4 示例：timestamp=1645877826760&aaa=123&bbb=456&apiSecret=api-secret&apikey=api-key
    let mut params = BTreeMap::new();
    params.insert(TIMESTAMP.to_string(), "1645877826760".into());
    params.insert("aaa".into(), "123".into());
    params.insert("bbb".into(), "456".into());
    let sign = get_sign("api-key", "api-secret", &params);
    assert_eq!(sign, "42f07b8eec6a9d1dfd0c1d8c587b2787");
  }

  #[test]
  fn plain_text_format() {
    let mut params = BTreeMap::new();
    params.insert(TIMESTAMP.to_string(), "1".into());
    params.insert("z".into(), "9".into());
    params.insert("a".into(), "2".into());
    let plain = get_plain_text("k", "s", "1", &params);
    assert_eq!(plain, "timestamp=1&a=2&z=9&apiSecret=s&apikey=k");
  }
}

#[tokio::main]
async fn main() -> e_utils::AnyResult<()> {
  DefaultSignSDK::example_request().await?;
  Ok(())
}
