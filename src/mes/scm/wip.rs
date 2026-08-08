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
      let s = match v {
        Value::String(s) => s.clone(),
        Value::Array(a) => a
          .iter()
          .filter_map(|x| x.as_str())
          .collect::<Vec<_>>()
          .join(","),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => other.to_string(),
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
  pub orgId: String,
  pub moId: String,
  pub moName: String,
  pub finMoName: String,
  pub lotNumber: String,
  pub crtTime: i64,
  pub updTime: i64,
  pub customerName: String,
  pub factoryCode: String,
  pub moProductionMode: String,
  pub scheduledStartDate: i64,
  pub moClassCode: String,
  pub status: String,
  pub statusName: String,
  pub moLineId: String,
  pub lineLotNumber: String,
  pub lineQuantity: i64,
  pub isBondedStores: String,
  pub itemId: String,
  pub itemCode: String,
  pub brand: String,
  pub saleModel: String,
  pub itemDesc: String,
  pub middleClassCode: String,
  pub itemSpec: String,
  pub pcbaProductClassMn: String,
  pub completedQuantity: i64,
  pub uncompletedQuantity: i64,
  pub completedDate: Option<i64>,
}

/// 查询SCM工单信息接口
pub async fn query_mo_factory(cfg: &Cfg, req: &MoFactoryReq) -> AnyResult<Vec<MoInfo>> {
  let url = format!("{}/wip/api/mo/factory", cfg.gateway);
  let body = serde_json::to_value(req)?;
  let params = sign_params(&body);
  let headers = sign::build_headers(&cfg.apikey, &cfg.api_secret, &params)?;
  let resp = reqwest::Client::new()
    .post(url)
    .headers(headers)
    .json(&body)
    .send()
    .await
    .map_err(|_| format!("SCM(query_mo_factory): 无法连接 {} 请检查网络!", cfg.gateway))?
    .json::<ScmResp<Vec<MoInfo>>>()
    .await?;
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
  pub moId: String,
  pub lotNumber: String,
  pub moLineId: String,
  pub lineLotNumber: String,
  pub reqLineId: String,
  pub operationSeqNo: String,
  pub itemCode: String,
  pub itemId: String,
  pub itemName: String,
  pub unitQty: f64,
  pub feedQty: f64,
  pub issuedQty: f64,
  pub updTime: i64,
}

/// 查询SCM投料单信息接口
pub async fn query_req_common(cfg: &Cfg, req: &FeedReq) -> AnyResult<Vec<FeedInfo>> {
  let url = format!("{}/wip/api/req/common/factory/query", cfg.gateway);
  let body = serde_json::to_value(req)?;
  let params = sign_params(&body);
  let headers = sign::build_headers(&cfg.apikey, &cfg.api_secret, &params)?;
  let resp = reqwest::Client::new()
    .post(url)
    .headers(headers)
    .json(&body)
    .send()
    .await
    .map_err(|_| format!("SCM(query_req_common): 无法连接 {} 请检查网络!", cfg.gateway))?
    .json::<ScmResp<Vec<FeedInfo>>>()
    .await?;
  match (resp.status.as_str(), resp.message.as_str(), resp.data) {
    ("0", "success", Some(data)) => Ok(data),
    (status, message, _) => Err(format!("SCM 查询投料单失败: status={} message={}", status, message).into()),
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
  fn sign_params_joins_arrays_with_comma() {
    let body = serde_json::json!({
      "lineLotNumberList": ["A", "B"],
      "lotNumberList": []
    });
    let params = sign_params(&body);
    assert_eq!(params.get("lineLotNumberList").unwrap(), "A,B");
    assert_eq!(params.get("lotNumberList").unwrap(), "");
    assert!(params.contains_key(TIMESTAMP));
  }
}
