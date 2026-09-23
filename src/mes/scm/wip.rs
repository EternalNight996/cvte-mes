#![allow(non_snake_case)]
//! SCM WIP 接口（文档：CVTE_SCM_WIP接口文档v1.0-查询工单投料信息 - MOKA-SK）
//!
//! - 查询SCM工单信息：POST /wip/api/mo/factory
//! - 查询SCM投料单信息：POST /wip/api/req/common/factory/query
//! 鉴权：MD5 签名（apikey / timestamp / sign），见 [`super::sign`]。
use std::collections::BTreeMap;

use e_utils::chrono::Utc;
use e_utils::AnyResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::sign::{self, TIMESTAMP};

/// SCM WIP 网关配置（测试网关默认 itgw-sit；正式网关由文档/现场提供）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cfg {
  /// 网关地址，如 https://itgw-sit.cvte.com/env-101/scm/scmwip/gw-wip
  pub gateway: String,
  /// 应用 ID（网关颁发）
  pub apikey: String,
  /// 应用密钥（网关颁发）
  pub api_secret: String,
}

impl Default for Cfg {
  fn default() -> Self {
    Self {
      gateway: "https://itgw-sit.cvte.com/env-101/scm/scmwip/gw-wip".into(),
      apikey: String::new(),
      api_secret: String::new(),
    }
  }
}

/// SCM 通用响应信封：{status, message, data}
#[derive(Debug, Deserialize)]
pub struct ScmResp<T> {
  #[serde(default)]
  pub status: String,
  #[serde(default)]
  pub message: String,
  #[serde(default)]
  pub data: Option<T>,
}

/// 签名业务参数：body 顶层字段 → 字符串（数组用逗号拼接）
fn sign_params(body: &Value) -> BTreeMap<String, String> {
  let mut map = BTreeMap::new();
  map.insert(
    TIMESTAMP.to_string(),
    (Utc::now().timestamp_millis() + 6_000_000).to_string(),
  );
  if let Some(obj) = body.as_object() {
    for (k, v) in obj {
      // 对齐官方 DefaultSignSDK.getPlainText：只有 String 类型参数参与签名串，
      // null 记为字面量 "null"，数组/数字/布尔一律跳过（放进去会导致与网关重算的签名不一致）。
      let s = match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        _ => continue,
      };
      map.insert(k.clone(), s);
    }
  }
  map
}

/// 查询SCM工单信息--工厂 入参
/// 注：文档备注 [生产小批次] 与 [工单最后更新时间起始/结束] 至少输入一项才可查询
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MoFactoryReq {
  /// 生产小批次（如 HEG-SKZBU23070075）
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lineLotNumber: Option<String>,
  /// 工单最后更新时间起始 yyyy-MM-dd HH:mm:ss
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lastUpdTimeStart: Option<String>,
  /// 工单最后更新时间结束 yyyy-MM-dd HH:mm:ss
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lastUpdTimeEnd: Option<String>,
}

/// 工单信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MoInfo {
  #[serde(default, deserialize_with = "null_to_default")]
  pub orgId: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub moId: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub moName: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub finMoName: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub lotNumber: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub crtTime: i64,
  #[serde(default, deserialize_with = "null_to_default")]
  pub updTime: i64,
  #[serde(default, deserialize_with = "null_to_default")]
  pub customerName: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub factoryCode: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub moProductionMode: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub scheduledStartDate: i64,
  #[serde(default, deserialize_with = "null_to_default")]
  pub moClassCode: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub status: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub statusName: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub moLineId: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub lineLotNumber: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub lineQuantity: i64,
  #[serde(default, deserialize_with = "null_to_default")]
  pub isBondedStores: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub itemId: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub itemCode: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub brand: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub saleModel: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub itemDesc: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub middleClassCode: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub itemSpec: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub pcbaProductClassMn: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub completedQuantity: i64,
  #[serde(default, deserialize_with = "null_to_default")]
  pub uncompletedQuantity: i64,
  #[serde(default, deserialize_with = "null_to_default")]
  pub completedDate: Option<i64>,
}

/// 查询SCM工单信息接口
pub async fn query_mo_factory(cfg: &Cfg, req: &MoFactoryReq) -> AnyResult<Vec<MoInfo>> {
  let url = format!("{}/wip/api/mo/factory", cfg.gateway);
  let body = serde_json::to_value(req)?;
  let params = sign_params(&body);
  let headers = sign::build_headers(&cfg.apikey, &cfg.api_secret, &params)?;
  let text = crate::mes::cmes::client()
    .post(url)
    .headers(headers)
    .json(&body)
    .send()
    .await
    .map_err(|_| format!("SCM(query_mo_factory): 无法连接 {} 请检查网络!", cfg.gateway))?
    .text()
    .await
    .map_err(|e| format!("SCM(query_mo_factory): 读取响应失败 ({e})"))?;
  let resp: ScmResp<Vec<MoInfo>> = serde_json::from_str(&text)
    .map_err(|e| format!("SCM(query_mo_factory): 响应非约定 JSON：{e}；原始响应：{}", clip(&text)))?;
  match (resp.status.as_str(), resp.message.as_str(), resp.data) {
    ("0", "success", Some(data)) => Ok(data),
    (status, message, _) => Err(format!("SCM 查询工单失败: status={} message={}", status, message).into()),
  }
}

/// 查询SCM投料单信息--工厂 入参
/// 注：文档备注 [生产小批次] 与 [工单生产批次] 至少输入一项才可查询
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeedReq {
  /// 生产小批次列表（如 ["HEG-SKZBU23070075"]）
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub lineLotNumberList: Vec<String>,
  /// 工单生产批次列表
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub lotNumberList: Vec<String>,
}

/// 投料单信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeedInfo {
  #[serde(default, deserialize_with = "null_to_default")]
  pub moId: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub lotNumber: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub moLineId: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub lineLotNumber: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub reqLineId: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub operationSeqNo: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub itemCode: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub itemId: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub itemName: String,
  #[serde(default, deserialize_with = "null_to_default")]
  pub unitQty: f64,
  #[serde(default, deserialize_with = "null_to_default")]
  pub feedQty: f64,
  #[serde(default, deserialize_with = "null_to_default")]
  pub issuedQty: f64,
  #[serde(default, deserialize_with = "null_to_default")]
  pub updTime: i64,
}

/// 查询SCM投料单信息接口
pub async fn query_req_common(cfg: &Cfg, req: &FeedReq) -> AnyResult<Vec<FeedInfo>> {
  let url = format!("{}/wip/api/req/common/factory/query", cfg.gateway);
  let body = serde_json::to_value(req)?;
  let params = sign_params(&body);
  let headers = sign::build_headers(&cfg.apikey, &cfg.api_secret, &params)?;
  let text = crate::mes::cmes::client()
    .post(url)
    .headers(headers)
    .json(&body)
    .send()
    .await
    .map_err(|_| format!("SCM(query_req_common): 无法连接 {} 请检查网络!", cfg.gateway))?
    .text()
    .await
    .map_err(|e| format!("SCM(query_req_common): 读取响应失败 ({e})"))?;
  let resp: ScmResp<Vec<FeedInfo>> = serde_json::from_str(&text)
    .map_err(|e| format!("SCM(query_req_common): 响应非约定 JSON：{e}；原始响应：{}", clip(&text)))?;
  match (resp.status.as_str(), resp.message.as_str(), resp.data) {
    ("0", "success", Some(data)) => Ok(data),
    (status, message, _) => Err(format!("SCM 查询投料单失败: status={} message={}", status, message).into()),
  }
}

/// 反序列化时把 null 当作该类型的默认值（SCM 网关会返回 null 字段，声明成非空 String 会解析失败）
fn null_to_default<'de, D, T>(d: D) -> Result<T, D::Error>
where
  D: serde::Deserializer<'de>,
  T: Default + serde::Deserialize<'de>,
{
  Ok(Option::<T>::deserialize(d)?.unwrap_or_default())
}

/// 截断原始响应便于排查（网关/nginx 常返回 HTML 错误页）
fn clip(text: &str) -> String {
  let t = text.trim();
  let cut: String = t.chars().take(300).collect();
  if t.chars().count() > 300 {
    format!("{cut}...")
  } else {
    cut
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mo_factory_req_serializes_skip_empty() {
    let req = MoFactoryReq {
      lineLotNumber: Some("HEG-SKZBU23070075".into()),
      ..Default::default()
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("lineLotNumber"));
    assert!(!json.contains("lastUpdTimeStart"));
  }

  #[test]
  fn feed_req_serializes_arrays() {
    let req = FeedReq {
      lineLotNumberList: vec!["HEG-SKZBU23070075".into()],
      ..Default::default()
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("lineLotNumberList"));
    assert!(!json.contains("lotNumberList"));
  }

  #[test]
  fn sign_params_skips_non_string_like_official_sdk() {
    // 官方 DefaultSignSDK 只把 String 参数纳入签名串：数组/数字/布尔跳过，null 记为 "null"
    let body = serde_json::json!({
      "lineLotNumberList": ["A", "B"],
      "lotNumberList": [],
      "qty": 5,
      "ok": true,
      "note": null,
      "lineLotNumber": "X"
    });
    let params = sign_params(&body);
    assert!(!params.contains_key("lineLotNumberList"));
    assert!(!params.contains_key("lotNumberList"));
    assert!(!params.contains_key("qty"));
    assert!(!params.contains_key("ok"));
    assert_eq!(params.get("note").unwrap(), "null");
    assert_eq!(params.get("lineLotNumber").unwrap(), "X");
    assert!(params.contains_key(TIMESTAMP));
  }
}