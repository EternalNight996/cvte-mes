#![allow(non_snake_case)]
//! ItAPI 接口（文档：获取生产文件和软件信息表接口信息）
//!
//! - 批量查询 CPLM 生产文件统一接口：POST /product/doc/common/batch_production_documents
//! - 根据生产批次ID或编号查询软件信息表生产文件列表：POST v1/soft_info/list/v2
//!
//! 文档未给出返参与鉴权细节：base_url 默认 itapi-portal.cvte.com，
//! apikey 鉴权头按 CVTE API 网关惯例提供（可留空），响应原样返回 JSON。
use e_utils::AnyResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// ItAPI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cfg {
  /// 门户/网关地址，默认 https://itapi-portal.cvte.com
  pub base_url: String,
  /// 应用 ID（鉴权头 apikey，网关颁发；文档未给出时留空）
  pub apikey: String,
}

impl Default for Cfg {
  fn default() -> Self {
    Self {
      base_url: "https://itapi-portal.cvte.com".into(),
      apikey: String::new(),
    }
  }
}

/// 批量查询 CPLM 生产文件 入参项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProdDocReq {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ckd: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub docSubClassList: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub docType: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub factoryCode: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub osSystem: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub prodNumber: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub productName: Option<String>,
}

/// 批量查询 CPLM 生产文件统一接口（返参按文档暂缺，原样返回 JSON）
pub async fn batch_production_documents(
  cfg: &Cfg,
  reqs: &[ProdDocReq],
) -> AnyResult<Value> {
  let url = format!(
    "{}/product/doc/common/batch_production_documents",
    cfg.base_url
  );
  let mut builder = reqwest::Client::new().post(url);
  if !cfg.apikey.is_empty() {
    builder = builder.header("apikey", &cfg.apikey);
  }
  builder
    .json(reqs)
    .send()
    .await
    .map_err(|_| format!("ItAPI(batch_production_documents): 无法连接 {} 请检查网络!", cfg.base_url))?
    .json::<Value>()
    .await
    .map_err(|e| format!("ItAPI(batch_production_documents): 响应解析失败: {e}").into())
}

/// 查询软件信息表 入参
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SoftInfoReq {
  /// 生产批次 ID 列表（如 ["38"]）
  pub lotNumbers: Vec<String>,
  /// 工单行 ID 列表（如 ["74"]）
  pub moLineIds: Vec<String>,
}

/// 根据生产批次ID或编号查询软件信息表生产文件列表（返参按文档暂缺，原样返回 JSON）
pub async fn soft_info_list_v2(cfg: &Cfg, req: &SoftInfoReq) -> AnyResult<Value> {
  let url = format!("{}/v1/soft_info/list/v2", cfg.base_url);
  let mut builder = reqwest::Client::new().post(url);
  if !cfg.apikey.is_empty() {
    builder = builder.header("apikey", &cfg.apikey);
  }
  builder
    .json(req)
    .send()
    .await
    .map_err(|_| format!("ItAPI(soft_info_list_v2): 无法连接 {} 请检查网络!", cfg.base_url))?
    .json::<Value>()
    .await
    .map_err(|e| format!("ItAPI(soft_info_list_v2): 响应解析失败: {e}").into())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn prod_doc_req_serializes_skip_empty() {
    let req = ProdDocReq {
      prodNumber: Some("001.001.0034469".into()),
      productName: Some("P22MB-BO1053B-M".into()),
      docType: Some("ALL".into()),
      ..Default::default()
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"prodNumber\":\"001.001.0034469\""));
    assert!(json.contains("\"docType\":\"ALL\""));
    assert!(!json.contains("osSystem"));
  }

  #[test]
  fn soft_info_req_serializes() {
    let req = SoftInfoReq {
      lotNumbers: vec!["38".into()],
      moLineIds: vec!["74".into()],
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"lotNumbers\":[\"38\"]"));
    assert!(json.contains("\"moLineIds\":[\"74\"]"));
  }
}
