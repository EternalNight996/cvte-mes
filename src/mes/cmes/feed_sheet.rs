#![allow(non_snake_case)]
//! CMES 前端 API：GetFeedSheetList（查询投料单，用户 2026-08-14 提供真实接口）
//!
//! ```text
//! GET {front_url}/api/services/app/ScanHead/GetFeedSheetList
//!   ?maxResultCount=60000&moLotNo={批次号或SN}&origin=0&parentId={...}&skipCount=0
//! 请求头：{tokenName}: {token}（token 由 login 获取，见 super::login）
//! ```
//!
//! 用途（PackPrint 后端包装站）：扫外箱条码 → 查该批次/SN 的投料单 →
//! 找物料描述含「包装箱」的物料 → 按单机用量换算本外箱装内箱数量（每箱数量）。
//!
//! ⚠ 字段名以接口实际返回为准：页面列名为 序号/物料编码/物料描述/批次号/单机用量/
//! 应发数量/已发数量/最后更新时间，本模块用多候选 key 兼容提取（见 `pick_*`）。
use reqwest::header;
use serde_json::Value;

use super::{header, Cfg, Response};

/// 投料单条目（PackPrint 建箱用；字段多候选提取，字段名待接口文档确认）
#[derive(Debug, Clone, Default)]
pub struct FeedSheetItem {
  /// 序号
  pub seq_no: Option<String>,
  /// 物料编码
  pub material_code: Option<String>,
  /// 物料描述（找「包装箱」关键词用）
  pub material_desc: Option<String>,
  /// 批次号
  pub batch_no: Option<String>,
  /// 单机用量（换算每箱数量用）
  pub unit_usage: Option<f64>,
  /// 应发数量
  pub plan_qty: Option<f64>,
  /// 已发数量
  pub issued_qty: Option<f64>,
  /// 最后更新时间
  pub last_update_time: Option<String>,
}

impl FeedSheetItem {
  /// 从 ABP 响应条目原始 JSON 提取（多候选 key，兼容不同字段命名）
  fn from_json(v: &Value) -> Self {
    Self {
      seq_no: pick_str(v, &["seqNo", "seq", "no", "序号"]),
      material_code: pick_str(v, &["materialCode", "itemCode", "materialNo", "物料编码"]),
      material_desc: pick_str(v, &["materialDesc", "itemDesc", "materialName", "物料描述"]),
      batch_no: pick_str(v, &["batchNo", "lotNo", "moLotNo", "批次号"]),
      unit_usage: pick_num(v, &["unitUsage", "usagePerUnit", "unitQty", "单机用量"]),
      plan_qty: pick_num(v, &["planQty", "shouldQty", "applyQty", "应发数量"]),
      issued_qty: pick_num(v, &["issuedQty", "sentQty", "releasedQty", "已发数量"]),
      last_update_time: pick_str(v, &["lastUpdateTime", "updTime", "最后更新时间"]),
    }
  }
}

/// 查询投料单（moLotNo 传批次号或 SN 均可，用户确认）
/// - `front_url`：前端 API 根，如 `http://heg-pc-cmes.gz.cvte.cn/frontApi/prod`
/// - `parent_id`：租户/组织标识（现场提供，如 3e5b8a05ea8b44efb3405d2cdf94663e）
pub async fn feed_sheet_list(
  cfg: &Cfg,
  token: &str,
  front_url: &str,
  parent_id: &str,
  mo_lot_no: &str,
) -> e_utils::AnyResult<Vec<FeedSheetItem>> {
  let url = format!("{}/api/services/app/ScanHead/GetFeedSheetList", front_url);
  let headers = header::HeaderMap::from_iter([header(&cfg.tokenName, token)?]);
  let response = reqwest::Client::new()
    .get(url)
    .headers(headers)
    .query(&[
      ("maxResultCount", "60000"),
      ("moLotNo", mo_lot_no),
      ("origin", "0"),
      ("parentId", parent_id),
      ("skipCount", "0"),
    ])
    .send()
    .await
    .map_err(|e| format!("CMES(feed_sheet_list)：无法链接 {front_url} 请检查网络状态! ({e})"))?
    .json::<Response<Option<Vec<Value>>>>()
    .await?;
  match (response.success, response.result, response.error) {
    (true, Some(rows), _) => Ok(rows.iter().map(FeedSheetItem::from_json).collect()),
    (_, _, Some(err)) => Err(err.message.into()),
    _ => Err("查询投料单失败，未知错误".into()),
  }
}

/// 多候选 key 提取字符串
fn pick_str(v: &Value, keys: &[&str]) -> Option<String> {
  keys.iter().find_map(|k| v.get(*k).and_then(|x| x.as_str().map(String::from)))
}

/// 多候选 key 提取数字（数值或数字字符串均可）
fn pick_num(v: &Value, keys: &[&str]) -> Option<f64> {
  keys.iter().find_map(|k| {
    v.get(*k).and_then(|x| {
      x.as_f64()
        .or_else(|| x.as_str().and_then(|s| s.trim().parse::<f64>().ok()))
    })
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn item_from_json_picks_known_fields() {
    let v: Value = serde_json::from_str(
      r#"{"seqNo":"1","materialCode":"016.001.0034190",
          "materialDesc":"外箱包装箱（长宽高 600x400x300）",
          "batchNo":"HEG-YKCBU2607034701","unitUsage":0.1,
          "planQty":100,"issuedQty":50,"lastUpdateTime":"2026-08-14 10:00:00"}"#,
    )
    .unwrap();
    let it = FeedSheetItem::from_json(&v);
    assert_eq!(it.material_code.as_deref(), Some("016.001.0034190"));
    assert!(it.material_desc.as_deref().unwrap_or("").contains("包装箱"));
    assert_eq!(it.unit_usage, Some(0.1));
    assert_eq!(it.batch_no.as_deref(), Some("HEG-YKCBU2607034701"));
  }

  #[test]
  fn item_from_json_accepts_alternative_keys() {
    // 接口字段名未定时：备选 key（itemCode / unitQty / 单机用量）也能提取
    let v: Value = serde_json::from_str(
      r#"{"no":"2","itemCode":"016.001.0034191","itemDesc":"内箱",
          "lotNo":"B-2","unitQty":"0.05","applyQty":200}"#,
    )
    .unwrap();
    let it = FeedSheetItem::from_json(&v);
    assert_eq!(it.seq_no.as_deref(), Some("2"));
    assert_eq!(it.material_code.as_deref(), Some("016.001.0034191"));
    assert_eq!(it.unit_usage, Some(0.05));
    assert_eq!(it.plan_qty, Some(200.0));
  }
}
