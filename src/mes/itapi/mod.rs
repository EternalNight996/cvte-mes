#![allow(non_snake_case)]
//! ItAPI 接口（文档：获取生产文件和软件信息表接口信息）
//!
//! - 批量查询 CPLM 生产文件统一接口：POST /product/doc/common/batch_production_documents
//! - 根据生产批次ID或编号查询软件信息表生产文件列表：POST v1/soft_info/list/v2
//!
//! 返参结构按参考设计（文档内截图）落实：普通 JSON 信封 {status, message, data:[...]}。
use e_utils::AnyResult;
use serde::{Deserialize, Serialize};

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

/// 批量查询 CPLM 生产文件 入参项（全部可选）
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

/// 截断原始响应，便于错误排查（未鉴权时 ItAPI 可能返回非本接口的 JSON/HTML）
fn clip(text: &str) -> String {
  let t = text.trim();
  let cut: String = t.chars().take(300).collect();
  if t.chars().count() > 300 {
    format!("{cut}...")
  } else {
    cut
  }
}

/// CPLM 生产文件（返参 data 项）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProdDoc {
  pub prodNumber: String,
  #[serde(default)]
  pub prodName: String,
  #[serde(default)]
  pub prodLifecycle: Option<String>,
  #[serde(default)]
  pub prodDept: Option<String>,
  #[serde(default)]
  pub prodId: String,
  #[serde(default)]
  pub docNumber: String,
  #[serde(default)]
  pub docName: String,
  #[serde(default)]
  pub docType: String,
  #[serde(default)]
  pub docVersion: Option<String>,
  #[serde(default)]
  pub docReleaseDate: Option<i64>,
  #[serde(default)]
  pub docReleaseDateStr: Option<String>,
  #[serde(default)]
  pub fileName: String,
  #[serde(default)]
  pub fileSize: Option<i64>,
  #[serde(default)]
  pub fileId: String,
  #[serde(default)]
  pub fileUrl: String,
  #[serde(default)]
  pub prodDocType: String,
  #[serde(default)]
  pub lotNumber: Option<String>,
}

/// CPLM 生产文件响应：{status, message, data}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProdDocResp {
  #[serde(default)]
  pub status: String,
  #[serde(default)]
  pub message: String,
  #[serde(default)]
  pub data: Option<Vec<ProdDoc>>,
}

/// 批量查询 CPLM 生产文件统一接口
pub async fn batch_production_documents(
  cfg: &Cfg,
  reqs: &[ProdDocReq],
) -> AnyResult<ProdDocResp> {
  let url = format!(
    "{}/product/doc/common/batch_production_documents",
    cfg.base_url
  );
  let mut builder = crate::mes::cmes::client().post(url);
  if !cfg.apikey.is_empty() {
    builder = builder.header("apikey", &cfg.apikey);
  }
  let text = builder
    .json(reqs)
    .send()
    .await
    .map_err(|_| format!("ItAPI(batch_production_documents): 无法连接 {} 请检查网络!", cfg.base_url))?
    .text()
    .await
    .map_err(|e| format!("ItAPI(batch_production_documents): 读取响应失败: {e}"))?;
  serde_json::from_str::<ProdDocResp>(&text).map_err(|e| {
    format!(
      "ItAPI(batch_production_documents): 响应解析失败: {e}；原始响应：{}",
      clip(&text)
    )
    .into()
  })
}

/// 查询软件信息表 入参
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SoftInfoReq {
  /// 生产批次 ID 列表（如 ["38"]）
  pub lotNumbers: Vec<String>,
  /// 工单行 ID 列表（如 ["74"]）
  pub moLineIds: Vec<String>,
}

/// 软件信息表生产文件（返参 data 项）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SoftInfo {
  #[serde(default)]
  pub batchNo: String,
  #[serde(default)]
  pub docNumber: String,
  #[serde(default)]
  pub docReleaseDate: String,
  #[serde(default)]
  pub docType: String,
  #[serde(default)]
  pub docVersion: String,
  #[serde(default)]
  pub fileHtmlUrl: String,
  #[serde(default)]
  pub fileId: String,
  #[serde(default)]
  pub fileName: String,
  #[serde(default)]
  pub fileSize: String,
  #[serde(default)]
  pub fileType: String,
  #[serde(default)]
  pub fileUrl: String,
  #[serde(default)]
  pub isDownloadedByFactory: String,
  #[serde(default)]
  pub moLineId: String,
  #[serde(default)]
  pub prodDept: String,
  #[serde(default)]
  pub prodDocType: String,
  #[serde(default)]
  pub prodLifecycle: String,
  #[serde(default)]
  pub prodName: String,
  #[serde(default)]
  pub swVersion: String,
  #[serde(default)]
  pub status: String,
}

/// 软件信息表响应：{status, message, data}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SoftInfoResp {
  #[serde(default)]
  pub status: String,
  #[serde(default)]
  pub message: String,
  #[serde(default)]
  pub data: Option<Vec<SoftInfo>>,
}

/// 根据生产批次ID或编号查询软件信息表生产文件列表
pub async fn soft_info_list_v2(cfg: &Cfg, req: &SoftInfoReq) -> AnyResult<SoftInfoResp> {
  let url = format!("{}/v1/soft_info/list/v2", cfg.base_url);
  let mut builder = crate::mes::cmes::client().post(url);
  if !cfg.apikey.is_empty() {
    builder = builder.header("apikey", &cfg.apikey);
  }
  let text = builder
    .json(req)
    .send()
    .await
    .map_err(|_| format!("ItAPI(soft_info_list_v2): 无法连接 {} 请检查网络!", cfg.base_url))?
    .text()
    .await
    .map_err(|e| format!("ItAPI(soft_info_list_v2): 读取响应失败: {e}"))?;
  serde_json::from_str::<SoftInfoResp>(&text).map_err(|e| {
    format!(
      "ItAPI(soft_info_list_v2): 响应解析失败: {e}；原始响应：{}",
      clip(&text)
    )
    .into()
  })
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

  #[test]
  fn prod_doc_resp_parses_reference_sample() {
    // 参考设计截图示例字段
    let text = r#"{"status":"0","message":"success","data":[{"prodNumber":"001.001.0034469","prodName":"P22MB-BO1053B-M","prodId":"207ebb0c70f849a19740df34bdfbfc9b","docNumber":"PDTØI-0046013","docName":"标签位置信息表","docType":"标签模板","docReleaseDate":1715831285000,"docReleaseDateStr":"2024-05-16 11:48:05","fileName":"能效贴纸.docx","fileSize":946483,"fileId":"fb80e28493ee490984683bbd36ccd9a6","fileUrl":"https://cplmobj-sit.gz.cvte.cn/...","prodDocType":"标签模板"}]}"#;
    let resp: ProdDocResp = serde_json::from_str(text).unwrap();
    assert_eq!(resp.status, "0");
    let doc = resp.data.unwrap();
    assert_eq!(doc.len(), 1);
    assert_eq!(doc[0].prodNumber, "001.001.0034469");
    assert_eq!(doc[0].fileSize, Some(946483));
  }

  #[test]
  fn soft_info_resp_parses_reference_sample() {
    let text = r#"{"data":[{"batchNo":"38","docNumber":"DOC-001","docType":"软件信息表","docVersion":"V1.0","fileUrl":"http://x/file.pdf","swVersion":"1.0.2","prodName":"P22MB-BO1053B-M"}]}"#;
    let resp: SoftInfoResp = serde_json::from_str(text).unwrap();
    let list = resp.data.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].batchNo, "38");
    assert_eq!(list[0].swVersion, "1.0.2");
  }
}
