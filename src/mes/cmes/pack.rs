#![allow(non_snake_case)]
//! CMES 前端 API：包装（Pack）—— 依据《CMES 对外接口对接标准》整理版补全
//!
//! ```text
//! GET {front_url}/api/services/pack/Pack/GetOtherListBySn?snCode={产品条码}
//! GET {front_url}/api/services/pack/Pack/GetSnPackFinishQty?moLotNo={批次}
//! 鉴权：x-auth-token: {Token}（现场前端以前缀配置 Cfg::tokenName 为准，见 [`super::session`]）
//! ```
//!
//! `front_url` 为前端 API 根（域名 + `/frontApi/prod`，如
//! `http://heg-pc-cmes.gz.cvte.cn/frontApi/prod`），与 [`super::feed_sheet`] 同一约定。
//!
//! 按条码查询用 [`get_other_list_by_sn`]（显式传 Token），
//! 需要 Token 缓存与 401 自动重试时用 [`Session::get_other_list_by_sn`]。
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::session::{self, Session};
use super::Cfg;

/// 按产品条码获取组件关联列表的结果行
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PackOtherItem {
  /// 组件关联记录主键
  pub id: String,
  /// 对应主条码 `PackDetailSub.Id`（多个主条码命中时用于区分来源）
  pub parentId: String,
  /// 组件基础数据 ID
  pub componentBasisId: String,
  /// 组件编码
  pub componentBasisCode: String,
  /// 组件名称
  pub componentBasisName: String,
  /// 组件条码
  pub snCode: String,
  /// ReelId
  pub reelId: String,
  /// 扫描数量（现有实现固定返回 1）
  pub scanQuantity: f64,
  /// 序号（按接口返回顺序从 1 开始递增）
  pub sort: i32,
  /// 是否完成
  pub isFinish: bool,
  /// 扫描时间
  pub creationTime: Option<String>,
  /// 扫描人
  pub creatorUserName: Option<String>,
  /// 最后更新时间
  pub lastModificationTime: Option<String>,
  /// 更新人
  pub lastModifierUserName: Option<String>,
  /// 物料编号
  pub partNo: String,
  /// 批次号（原批次，来源主条码 `PackDetailSub.OrderNo`）
  pub batchNo: String,
}

/// 按批次获取 SN 包装完工数的结果行（每行 = 一个（批次, 类型）组合）
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SnPackFinishQty {
  /// 生产批次
  pub batchNo: String,
  /// 类型：整机（来源 `CMP_FACTORY`）/ 部件（来源 `CMP_GW`）
  #[serde(rename = "type")]
  pub r#type: String,
  /// 该（批次, 类型）已包装完工 SN 数（按 SnCode 去重，非行数）
  pub finishQty: i64,
}

/// 按产品条码获取组件关联列表（GetOtherListBySn）
///
/// `snCode` 去空白后为空时不发起请求，直接返回参数错误（对接标准 8.4）。
pub async fn get_other_list_by_sn(
  cfg: &Cfg,
  token: &str,
  front_url: &str,
  sn_code: &str,
) -> e_utils::AnyResult<Vec<PackOtherItem>> {
  let sn = check_sn(sn_code)?;
  let url = format!("{}/api/services/pack/Pack/GetOtherListBySn", root(front_url));
  let value = session::get_front_json_with_token(cfg, token, &url, &[("snCode", sn)]).await?;
  parse_other_items(&value)
}

/// 按批次获取 SN 包装完工数（GetSnPackFinishQty）
///
/// 服务端按 `moLotNo` 维度限流（默认 10 秒窗口 30 次），请勿在循环里高频重试。
/// `moLotNo` 去空白后为空时不发起请求，直接返回参数错误。
pub async fn get_sn_pack_finish_qty(
  cfg: &Cfg,
  token: &str,
  front_url: &str,
  mo_lot_no: &str,
) -> e_utils::AnyResult<Vec<SnPackFinishQty>> {
  let lot = check_lot(mo_lot_no)?;
  let url = format!("{}/api/services/pack/Pack/GetSnPackFinishQty", root(front_url));
  let value = session::get_front_json_with_token(cfg, token, &url, &[("moLotNo", lot)]).await?;
  parse_finish_rows(&value)
}

impl Session {
  /// 会话版：按产品条码获取组件关联列表（Token 缓存 + 失效自动重取并重试一次）
  pub async fn get_other_list_by_sn(&self, front_url: &str, sn_code: &str) -> e_utils::AnyResult<Vec<PackOtherItem>> {
    let sn = check_sn(sn_code)?;
    let url = format!("{}/api/services/pack/Pack/GetOtherListBySn", root(front_url));
    let value = self.get_front_json(&url, &[("snCode", sn)]).await?;
    parse_other_items(&value)
  }

  /// 会话版：按批次获取 SN 包装完工数（Token 缓存 + 失效自动重取并重试一次）
  pub async fn get_sn_pack_finish_qty(&self, front_url: &str, mo_lot_no: &str) -> e_utils::AnyResult<Vec<SnPackFinishQty>> {
    let lot = check_lot(mo_lot_no)?;
    let url = format!("{}/api/services/pack/Pack/GetSnPackFinishQty", root(front_url));
    let value = self.get_front_json(&url, &[("moLotNo", lot)]).await?;
    parse_finish_rows(&value)
  }
}

/// 前端 API 根去首尾空白与尾斜杠（避免拼出 `//api/...`）
fn root(front_url: &str) -> &str {
  front_url.trim().trim_end_matches('/')
}

/// 入参校验：`snCode` 去除首尾空白后不能为空（对接标准 8.1）
fn check_sn(sn_code: &str) -> e_utils::AnyResult<&str> {
  let sn = sn_code.trim();
  if sn.is_empty() {
    return Err("CMES(GetOtherListBySn)：snCode 不能为空或仅空白（参数错误，未发起请求）".into());
  }
  Ok(sn)
}

/// 入参校验：`moLotNo` 去除首尾空白后不能为空（对接标准 9.1）
fn check_lot(mo_lot_no: &str) -> e_utils::AnyResult<&str> {
  let lot = mo_lot_no.trim();
  if lot.is_empty() {
    return Err("CMES(GetSnPackFinishQty)：moLotNo 不能为空或仅空白（参数错误，未发起请求）".into());
  }
  Ok(lot)
}

/// 解析 `result` 为组件关联列表（无数据时服务端固定返回空数组；`null` 亦按空列表处理）
fn parse_other_items(value: &Value) -> e_utils::AnyResult<Vec<PackOtherItem>> {
  match value {
    Value::Null => Ok(Vec::new()),
    Value::Array(rows) => {
      let mut items = Vec::with_capacity(rows.len());
      for row in rows {
        let item = serde_json::from_value(row.clone()).map_err(|e| format!("CMES(GetOtherListBySn)：解析组件明细失败：{e}"))?;
        items.push(item);
      }
      Ok(items)
    }
    other => Err(format!("CMES(GetOtherListBySn)：result 应为数组，实际为 {other}").into()),
  }
}

/// 解析 `result` 为包装完工数列表（无数据时为 `[]`）
fn parse_finish_rows(value: &Value) -> e_utils::AnyResult<Vec<SnPackFinishQty>> {
  match value {
    Value::Null => Ok(Vec::new()),
    Value::Array(rows) => {
      let mut items = Vec::with_capacity(rows.len());
      for row in rows {
        let item = serde_json::from_value(row.clone()).map_err(|e| format!("CMES(GetSnPackFinishQty)：解析完工数失败：{e}"))?;
        items.push(item);
      }
      Ok(items)
    }
    other => Err(format!("CMES(GetSnPackFinishQty)：result 应为数组，实际为 {other}").into()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_other_items_reads_documented_fields() {
    let value: Value = serde_json::from_str(
      r#"[{
        "id":"a1","parentId":"p1","componentBasisId":"c1","componentBasisCode":"CMP_GW",
        "componentBasisName":"主板","snCode":"U23070429-0A03693","reelId":"U23070429-0A03693",
        "scanQuantity":1,"sort":1,"isFinish":true,"creationTime":"2026-09-07T10:20:30",
        "creatorUserName":"操作人","lastModificationTime":null,"lastModifierUserName":null,
        "partNo":"002.002.0223135","batchNo":"HEG-YKCBU2607034701"
      }]"#,
    )
    .unwrap();
    let rows = parse_other_items(&value).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].componentBasisCode, "CMP_GW");
    assert_eq!(rows[0].snCode, "U23070429-0A03693");
    assert_eq!(rows[0].scanQuantity, 1.0);
    assert_eq!(rows[0].sort, 1);
    assert!(rows[0].isFinish);
    assert_eq!(rows[0].batchNo, "HEG-YKCBU2607034701");
    assert_eq!(rows[0].lastModificationTime, None);
  }

  #[test]
  fn parse_other_items_treats_empty_and_null_as_no_data() {
    assert!(parse_other_items(&serde_json::json!([])).unwrap().is_empty());
    assert!(parse_other_items(&Value::Null).unwrap().is_empty());
    assert!(parse_other_items(&serde_json::json!({"result":"unexpected"})).is_err());
  }

  #[test]
  fn parse_other_items_tolerates_missing_optional_fields() {
    let rows = parse_other_items(&serde_json::json!([{"snCode":"SN1"}])).unwrap();
    assert_eq!(rows[0].snCode, "SN1");
    assert_eq!(rows[0].partNo, "");
    assert_eq!(rows[0].creatorUserName, None);
  }

  #[test]
  fn parse_finish_rows_reads_type_and_qty() {
    let value: Value = serde_json::from_str(
      r#"[{"batchNo":"LOT001","type":"整机","finishQty":120},
          {"batchNo":"LOT001","type":"部件","finishQty":40}]"#,
    )
    .unwrap();
    let rows = parse_finish_rows(&value).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].r#type, "整机");
    assert_eq!(rows[0].finishQty, 120);
    assert_eq!(rows[1].r#type, "部件");
    assert_eq!(rows[1].finishQty, 40);
  }

  #[test]
  fn blank_arguments_are_rejected_before_request() {
    assert!(check_sn("   ").is_err());
    assert!(check_lot("").is_err());
    assert_eq!(check_sn(" SN001 ").unwrap(), "SN001");
    assert_eq!(check_lot(" LOT001 ").unwrap(), "LOT001");
  }

  #[test]
  fn root_trims_trailing_slash() {
    assert_eq!(root("http://host/frontApi/prod/"), "http://host/frontApi/prod");
    assert_eq!(root(" http://host/frontApi/prod "), "http://host/frontApi/prod");
  }
}
