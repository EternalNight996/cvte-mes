#![allow(non_snake_case)]
//! CMES 前端 API：基础数据（bas）—— 依据《CMES 对外接口对接标准 V1.1（2026-10-14）》第 10~12 章
//!
//! ```text
//! GET {front_url}/api/services/bas/FeedSheetOrder/GetFeedSheetByContainerName?containerName={条码}
//! GET {front_url}/api/services/bas/ScanHistory/GetStationPassQty?moLotNo={批次}[&stationCodes=B10,B20]
//! GET {front_url}/api/services/bas/SerialNoCache/GetInfoBySn?sn={条码}[&isIncludeOrder=true][&componentBasisCode={类型}]
//! 鉴权：x-auth-token: {Token}（现场前端以前缀配置 Cfg::tokenName 为准，见 [`super::session`]）
//! ```
//!
//! `front_url` 为前端 API 根（域名 + `/frontApi/prod`，如
//! `http://heg-pc-cmes.gz.cvte.cn/frontApi/prod`），与 [`super::pack`] / [`super::feed_sheet`] 同一约定。
//!
//! | 章 | 接口 | 说明 | 限流 |
//! |---|---|---|---|
//! | 10 | [`feed_sheet_by_container_name`] | 按条码取所属批次的投料单物料行（应发数 > 0） | 10 秒 30 次/条码 |
//! | 11 | [`station_pass_qty`] | 按批次取各工步过站数量（整机/部件分行） | 10 秒 30 次/批次 |
//! | 12 | [`serial_no_info`] | 按条码取条码详细信息（**条码不存在返回 `None`，非异常**） | 无 |
//!
//! 需要 Token 缓存与 401 自动重试时用 [`Session`] 上的同名方法。
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::session::{self, Session};
use super::Cfg;

/// 第 10 章：投料单物料行（`GetFeedSheetByContainerName`）
///
/// 服务端只返回应发数量 > 0 的行，并按物料编码倒序；无数据时 `result` 为空数组。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct FeedSheetPart {
  /// 物料 Id
  pub partId: i64,
  /// 物料编码
  pub partNo: String,
  /// 物料描述
  pub partName: String,
  /// 应发数量 / 需求数量
  pub needNum: f64,
  /// 单位用量
  pub quantityPerAssembly: f64,
  /// 是否启用批次管控
  pub isLotControl: bool,
  /// 批次号（条码所属批次）
  pub moLotNo: String,
}

/// 第 11 章：工步过站数量行（`GetStationPassQty`，每行 = 一个（工步, 条码类型）组合）
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct StationPassRow {
  /// 生产批次
  pub moLotNo: String,
  /// 工步 Id
  pub stationId: String,
  /// 工步编码；**工步缓存未命中时为 `null`**
  pub stationCode: Option<String>,
  /// 工步名称
  pub stationName: String,
  /// 工步顺序（结果按此升序；同一工步整机在前、部件在后）
  pub sortNo: i64,
  /// 批次数量（计划数），每个工步行均携带
  pub moQty: i64,
  /// 整机（工厂条码）/ 部件（高温条码）
  #[serde(rename = "type")]
  pub r#type: String,
  /// 该（批次, 工步, 类型）的过站数量（按条码去重，非记录行数）
  pub passQty: i64,
}

/// 第 12 章：条码详细信息（`GetInfoBySn`）
///
/// 标注 `isIncludeOrder=false 时为空` 的字段统一用 `Option`：服务端可能返回 `null` 或省略该字段。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SerialNoInfo {
  /// 条码流水 Id
  pub id: String,
  /// 所属单据 Id
  pub orderId: String,
  /// 单据号
  pub orderNo: String,
  /// 批次号（与单据号一致）
  pub moLotNo: String,
  /// 单据数量（计划数）
  pub orderQty: f64,
  /// 单据数量（兼容字段，与 `orderQty` 一致；文档标 int，实际与 `orderQty` 同为数量故按小数接收）
  pub qty: f64,
  /// 组件 Id
  pub componentBasisId: String,
  /// 条码类型编码
  pub componentBasisCode: String,
  /// 条码（与 `snCode` 值相同）
  pub sn: String,
  /// 条码（`sn` 的兼容字段）
  pub snCode: String,
  /// ReelId
  pub reelId: String,
  /// 流水序号
  pub snNum: i64,
  /// 条码容量
  pub snQty: f64,
  /// 内箱份数（外箱条码）
  pub innerNum: f64,
  /// 组件类型
  pub subType: String,
  /// 物料 Id；`isIncludeOrder=false` 时为空
  pub partId: Option<i64>,
  /// 产品编码；`isIncludeOrder=false` 时为空
  pub partNo: Option<String>,
  /// 物料描述；`isIncludeOrder=false` 时为空
  pub partName: Option<String>,
  /// 产品简称；`isIncludeOrder=false` 时为空
  pub productAbName: Option<String>,
  /// 产品型号；`isIncludeOrder=false` 时为空
  pub productModel: Option<String>,
  /// 销售型号；`isIncludeOrder=false` 时为空
  pub sellModel: Option<String>,
  /// 电气参数；`isIncludeOrder=false` 时为空
  pub electricParam: Option<String>,
  /// 是否为拼板高温条码
  pub isPuzzle: bool,
  /// 条码规则 Id
  pub serialNoRuleSetId: String,
  /// 条码生成记录 Id
  pub serialNoHeadId: String,
}

/// 第 10 章：扫描条码获取批次投料单物料信息（GetFeedSheetByContainerName）
///
/// - `container_name`：扫描条码（产品条码 / 容器条码），服务端据其反查所属批次号
/// - 服务端按 `containerName` 维度限流（默认 10 秒窗口 30 次），请勿在循环里高频重试
/// - `containerName` 去空白后为空时不发起请求，直接返回参数错误（标准 10.4）
pub async fn feed_sheet_by_container_name(
  cfg: &Cfg,
  token: &str,
  front_url: &str,
  container_name: &str,
) -> e_utils::AnyResult<Vec<FeedSheetPart>> {
  let name = check_container(container_name)?;
  let url = format!("{}/api/services/bas/FeedSheetOrder/GetFeedSheetByContainerName", root(front_url));
  let value = session::get_json_with_token(cfg, token, &url, &[("containerName", name)]).await?;
  parse_feed_sheet_parts(&value)
}

/// 第 11 章：按批次获取各工步过站数量（GetStationPassQty）
///
/// - `station_codes`：工步编码，多个用英文逗号分隔（如 `B10,B20`），单次最多 50 个；
///   传 `None` 或空白则统计该批次全部工步
/// - `moLotNo` 去空白后为空、超过 100 位，或 `stationCodes` 超过 50 个时不发起请求（标准 11.5）
/// - 服务端按 `moLotNo` 维度限流（默认 10 秒窗口 30 次）
pub async fn station_pass_qty(
  cfg: &Cfg,
  token: &str,
  front_url: &str,
  mo_lot_no: &str,
  station_codes: Option<&str>,
) -> e_utils::AnyResult<Vec<StationPassRow>> {
  let lot = check_lot(mo_lot_no)?;
  let codes = check_station_codes(station_codes)?;
  let url = format!("{}/api/services/bas/ScanHistory/GetStationPassQty", root(front_url));
  let mut query: Vec<(&str, &str)> = vec![("moLotNo", lot)];
  if let Some(ref codes) = codes {
    query.push(("stationCodes", codes.as_str()));
  }
  let value = session::get_json_with_token(cfg, token, &url, &query).await?;
  parse_station_rows(&value)
}

/// 第 12 章：按条码获取条码详细信息（GetInfoBySn）
///
/// - `is_include_order`：是否附带所属单据与产品/物料信息，服务端默认 `true`；传 `None` 即用服务端默认
/// - `component_basis_code`：仅查询该条码类型的记录；传 `None` 即不过滤
/// - **条码不存在时服务端仍返回 `success: true` 且 `result: null`，此时返回 `Ok(None)`**——
///   条码不存在不属于异常，调用方应以本函数返回的 `Option` 判断，不要按错误处理（标准 12.2）
pub async fn serial_no_info(
  cfg: &Cfg,
  token: &str,
  front_url: &str,
  sn: &str,
  is_include_order: Option<bool>,
  component_basis_code: Option<&str>,
) -> e_utils::AnyResult<Option<SerialNoInfo>> {
  let sn = check_sn(sn)?;
  let url = format!("{}/api/services/bas/SerialNoCache/GetInfoBySn", root(front_url));
  let mut query: Vec<(&str, &str)> = vec![("sn", sn)];
  if let Some(flag) = is_include_order {
    query.push(("isIncludeOrder", if flag { "true" } else { "false" }));
  }
  if let Some(code) = component_basis_code.map(str::trim).filter(|s| !s.is_empty()) {
    query.push(("componentBasisCode", code));
  }
  let value = session::get_json_with_token(cfg, token, &url, &query).await?;
  parse_serial_info(&value)
}

impl Session {
  /// 会话版：按条码取批次投料单物料信息（Token 缓存 + 失效自动重取并重试一次）
  pub async fn feed_sheet_by_container_name(
    &self,
    front_url: &str,
    container_name: &str,
  ) -> e_utils::AnyResult<Vec<FeedSheetPart>> {
    let name = check_container(container_name)?;
    let url = format!("{}/api/services/bas/FeedSheetOrder/GetFeedSheetByContainerName", root(front_url));
    let value = self.get_json(&url, &[("containerName", name)]).await?;
    parse_feed_sheet_parts(&value)
  }

  /// 会话版：按批次取各工步过站数量（Token 缓存 + 失效自动重取并重试一次）
  pub async fn station_pass_qty(
    &self,
    front_url: &str,
    mo_lot_no: &str,
    station_codes: Option<&str>,
  ) -> e_utils::AnyResult<Vec<StationPassRow>> {
    let lot = check_lot(mo_lot_no)?;
    let codes = check_station_codes(station_codes)?;
    let url = format!("{}/api/services/bas/ScanHistory/GetStationPassQty", root(front_url));
    let mut query: Vec<(&str, &str)> = vec![("moLotNo", lot)];
    if let Some(ref codes) = codes {
      query.push(("stationCodes", codes.as_str()));
    }
    let value = self.get_json(&url, &query).await?;
    parse_station_rows(&value)
  }

  /// 会话版：按条码取条码详细信息（**条码不存在返回 `None`**）
  pub async fn serial_no_info(
    &self,
    front_url: &str,
    sn: &str,
    is_include_order: Option<bool>,
    component_basis_code: Option<&str>,
  ) -> e_utils::AnyResult<Option<SerialNoInfo>> {
    let sn = check_sn(sn)?;
    let url = format!("{}/api/services/bas/SerialNoCache/GetInfoBySn", root(front_url));
    let mut query: Vec<(&str, &str)> = vec![("sn", sn)];
    if let Some(flag) = is_include_order {
      query.push(("isIncludeOrder", if flag { "true" } else { "false" }));
    }
    if let Some(code) = component_basis_code.map(str::trim).filter(|s| !s.is_empty()) {
      query.push(("componentBasisCode", code));
    }
    let value = self.get_json(&url, &query).await?;
    parse_serial_info(&value)
  }
}

/// 前端 API 根去首尾空白与尾斜杠（避免拼出 `//api/...`）
fn root(front_url: &str) -> &str {
  front_url.trim().trim_end_matches('/')
}

/// 递归剔除对象中的 `null` 值后再反序列化。
///
/// `#[serde(default)]` 只兜「字段缺失」，**不兜「字段为 null」**。第 12 章
/// `isIncludeOrder=false` 时多个字段以 `null` 返回（文档的逐字段「为空」标注并不完整，
/// 实测单据类字段 `orderId` / `orderNo` / `moLotNo` 亦会为 `null`），
/// 任何一个 `null` 都会让整行解析失败。故统一按「`null` ≡ 缺失 ≡ 默认值」处理。
fn without_nulls(value: &Value) -> Value {
  match value {
    Value::Object(map) => Value::Object(
      map
        .iter()
        .filter(|(_, v)| !v.is_null())
        .map(|(k, v)| (k.clone(), without_nulls(v)))
        .collect(),
    ),
    Value::Array(rows) => Value::Array(rows.iter().map(without_nulls).collect()),
    other => other.clone(),
  }
}

/// 入参校验：`containerName` 去空白后不能为空（对接标准 10.1 / 10.4）
fn check_container(container_name: &str) -> e_utils::AnyResult<&str> {
  let name = container_name.trim();
  if name.is_empty() {
    return Err("CMES(GetFeedSheetByContainerName)：containerName 不能为空或仅空白（参数错误，未发起请求）".into());
  }
  Ok(name)
}

/// 入参校验：`moLotNo` 去空白后不能为空且不超过 100 位（对接标准 11.2 / 11.5）
fn check_lot(mo_lot_no: &str) -> e_utils::AnyResult<&str> {
  let lot = mo_lot_no.trim();
  if lot.is_empty() {
    return Err("CMES(GetStationPassQty)：moLotNo 不能为空或仅空白（参数错误，未发起请求）".into());
  }
  if lot.chars().count() > 100 {
    return Err(format!(
      "CMES(GetStationPassQty)：moLotNo 长度 {} 超过 100 位（参数错误，未发起请求）",
      lot.chars().count()
    )
    .into());
  }
  Ok(lot)
}

/// 入参校验：`stationCodes` 逗号分隔、去空白后每段非空、单次最多 50 个（对接标准 11.2 / 11.5）
///
/// 返回 `None` 表示不传该参数（统计全部工步）。返回值已做「去空白 + 重新以英文逗号连接」归一化，
/// 避免 `B10, B20` 这类带空格的写法被服务端当成不存在的工步编码。
fn check_station_codes(station_codes: Option<&str>) -> e_utils::AnyResult<Option<String>> {
  let Some(raw) = station_codes else {
    return Ok(None);
  };
  let codes: Vec<&str> = raw.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
  if codes.is_empty() {
    return Ok(None);
  }
  if codes.len() > 50 {
    return Err(format!(
      "CMES(GetStationPassQty)：stationCodes 共 {} 个，超过单次上限 50 个（参数错误，未发起请求）",
      codes.len()
    )
    .into());
  }
  Ok(Some(codes.join(",")))
}

/// 入参校验：`sn` 去空白后不能为空（对接标准 12.1 / 12.4）
fn check_sn(sn: &str) -> e_utils::AnyResult<&str> {
  let sn = sn.trim();
  if sn.is_empty() {
    return Err("CMES(GetInfoBySn)：sn 不能为空或仅空白（参数错误，未发起请求）".into());
  }
  Ok(sn)
}

/// 解析 `result` 为投料行数组（无数据时服务端固定返回空数组；`null` 亦按空列表处理）
fn parse_feed_sheet_parts(value: &Value) -> e_utils::AnyResult<Vec<FeedSheetPart>> {
  match value {
    Value::Null => Ok(Vec::new()),
    Value::Array(rows) => {
      let mut items = Vec::with_capacity(rows.len());
      for row in rows {
        let item = serde_json::from_value(without_nulls(row))
          .map_err(|e| format!("CMES(GetFeedSheetByContainerName)：解析投料行失败：{e}；原始行：{row}"))?;
        items.push(item);
      }
      Ok(items)
    }
    other => Err(format!("CMES(GetFeedSheetByContainerName)：result 应为数组，实际为 {other}").into()),
  }
}

/// 解析 `result` 为工步过站行数组（无数据时为 `[]`）
fn parse_station_rows(value: &Value) -> e_utils::AnyResult<Vec<StationPassRow>> {
  match value {
    Value::Null => Ok(Vec::new()),
    Value::Array(rows) => {
      let mut items = Vec::with_capacity(rows.len());
      for row in rows {
        let item = serde_json::from_value(without_nulls(row))
          .map_err(|e| format!("CMES(GetStationPassQty)：解析过站行失败：{e}；原始行：{row}"))?;
        items.push(item);
      }
      Ok(items)
    }
    other => Err(format!("CMES(GetStationPassQty)：result 应为数组，实际为 {other}").into()),
  }
}

/// 解析 `result` 为条码信息（**`null` = 条码不存在，返回 `Ok(None)`，不得当异常**，对接标准 12.2）
fn parse_serial_info(value: &Value) -> e_utils::AnyResult<Option<SerialNoInfo>> {
  match value {
    Value::Null => Ok(None),
    Value::Object(_) => {
      let info = serde_json::from_value(without_nulls(value))
        .map_err(|e| format!("CMES(GetInfoBySn)：解析条码信息失败：{e}；原始响应：{value}"))?;
      Ok(Some(info))
    }
    other => Err(format!("CMES(GetInfoBySn)：result 应为对象或 null，实际为 {other}").into()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_feed_sheet_parts_reads_documented_fields() {
    let value: Value = serde_json::from_str(
      r#"[{"partId":10001,"partNo":"006.xxx","partName":"物料描述","needNum":12.0,
           "quantityPerAssembly":1.0,"isLotControl":true,"moLotNo":"LOT001"}]"#,
    )
    .unwrap();
    let rows = parse_feed_sheet_parts(&value).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].partId, 10001);
    assert_eq!(rows[0].partNo, "006.xxx");
    assert_eq!(rows[0].needNum, 12.0);
    assert_eq!(rows[0].quantityPerAssembly, 1.0);
    assert!(rows[0].isLotControl);
    assert_eq!(rows[0].moLotNo, "LOT001");
  }

  #[test]
  fn parse_feed_sheet_parts_treats_empty_and_null_as_no_data() {
    assert!(parse_feed_sheet_parts(&serde_json::json!([])).unwrap().is_empty());
    assert!(parse_feed_sheet_parts(&Value::Null).unwrap().is_empty());
    assert!(parse_feed_sheet_parts(&serde_json::json!({"result":"unexpected"})).is_err());
  }

  #[test]
  fn parse_station_rows_keeps_null_station_code() {
    // 标准 11.4：工步缓存未命中时 stationCode 为 null
    let value: Value = serde_json::from_str(
      r#"[{"moLotNo":"LOT001","stationId":"8f2b","stationCode":"B10","stationName":"组装",
            "sortNo":10,"moQty":200,"type":"整机","passQty":120},
           {"moLotNo":"LOT001","stationId":"9a3c","stationCode":null,"stationName":"测试",
            "sortNo":20,"moQty":200,"type":"部件","passQty":40}]"#,
    )
    .unwrap();
    let rows = parse_station_rows(&value).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].stationCode.as_deref(), Some("B10"));
    assert_eq!(rows[0].r#type, "整机");
    assert_eq!(rows[0].passQty, 120);
    assert_eq!(rows[0].moQty, 200);
    assert_eq!(rows[1].stationCode, None);
    assert_eq!(rows[1].r#type, "部件");
  }

  #[test]
  fn parse_station_rows_treats_null_as_no_data() {
    assert!(parse_station_rows(&Value::Null).unwrap().is_empty());
    assert!(parse_station_rows(&serde_json::json!([])).unwrap().is_empty());
  }

  #[test]
  fn parse_serial_info_null_means_barcode_not_found() {
    // 标准 12.2 的核心约定：条码不存在时 success 仍为 true 且 result 为 null —— 不是异常
    assert_eq!(parse_serial_info(&Value::Null).unwrap(), None);
  }

  #[test]
  fn parse_serial_info_reads_documented_fields() {
    let value: Value = serde_json::from_str(
      r#"{"id":"3fa85f64","orderId":"5b21c0de","orderNo":"PO20261001001","moLotNo":"LOT001",
          "orderQty":200.0,"qty":200,"componentBasisId":"9c2d5e1a","componentBasisCode":"CMP_FACTORY",
          "sn":"SN20261001001","snCode":"SN20261001001","reelId":"REEL001","snNum":1,"snQty":1.0,
          "innerNum":4.0,"subType":"组件类型编码","partId":10001,"partNo":"P-0001",
          "partName":"物料描述","productAbName":"产品简称","productModel":"产品型号A",
          "sellModel":"销售型号A","electricParam":"电气参数","isPuzzle":false,
          "serialNoRuleSetId":"rule-1","serialNoHeadId":"head-1"}"#,
    )
    .unwrap();
    let info = parse_serial_info(&value).unwrap().expect("应解析出条码信息");
    assert_eq!(info.id, "3fa85f64");
    assert_eq!(info.orderNo, "PO20261001001");
    assert_eq!(info.orderQty, 200.0);
    assert_eq!(info.qty, 200.0);
    assert_eq!(info.sn, info.snCode);
    assert_eq!(info.snNum, 1);
    assert_eq!(info.innerNum, 4.0);
    assert_eq!(info.partId, Some(10001));
    assert_eq!(info.partNo.as_deref(), Some("P-0001"));
    assert!(!info.isPuzzle);
  }

  #[test]
  fn parse_serial_info_tolerates_is_include_order_false() {
    // isIncludeOrder=false：单据/产品/物料字段为空。
    // 注意：实测（本地保真 mock）单据类字段 orderId/orderNo/moLotNo 也会为 null，
    // 文档的逐字段「为空」标注并不完整，故 null ≡ 缺失 ≡ 默认值。
    let value = serde_json::json!({
      "id":"1","sn":"SN1","snCode":"SN1",
      "orderId":null,"orderNo":null,"moLotNo":null,"orderQty":null,"qty":null,
      "reelId":null,"snNum":null,"snQty":null,"innerNum":null,"subType":null,
      "isPuzzle":null,"serialNoRuleSetId":null,"serialNoHeadId":null,
      "partId":null,"partNo":null,"partName":null,"productAbName":null,
      "productModel":null,"sellModel":null,"electricParam":null
    });
    let info = parse_serial_info(&value).unwrap().unwrap();
    assert_eq!(info.orderId, "");
    assert_eq!(info.orderNo, "");
    assert_eq!(info.moLotNo, "");
    assert_eq!(info.orderQty, 0.0);
    assert_eq!(info.snNum, 0);
    assert!(!info.isPuzzle);
    assert_eq!(info.partId, None);
    assert_eq!(info.partNo, None);
    assert_eq!(info.sn, "SN1");
  }

  #[test]
  fn without_nulls_strips_nested_nulls_in_objects_and_arrays() {
    let value = serde_json::json!({
      "keep": 1, "drop": null,
      "obj": { "inner_keep": "x", "inner_drop": null },
      "arr": [ { "a": 1, "b": null } ]
    });
    let cleaned = without_nulls(&value);
    assert_eq!(cleaned["keep"], 1);
    assert!(cleaned.get("drop").is_none());
    assert!(cleaned["obj"].get("inner_drop").is_none());
    assert!(cleaned["arr"][0].get("b").is_none());
  }

  #[test]
  fn parse_serial_info_rejects_non_object_non_null() {
    assert!(parse_serial_info(&serde_json::json!([1, 2])).is_err());
  }

  #[test]
  fn blank_and_overlong_arguments_are_rejected_before_request() {
    assert!(check_container("   ").is_err());
    assert!(check_sn("").is_err());
    assert!(check_lot("  ").is_err());
    assert_eq!(check_container(" SN1 ").unwrap(), "SN1");
    assert_eq!(check_sn(" SN1 ").unwrap(), "SN1");
    assert_eq!(check_lot(" LOT001 ").unwrap(), "LOT001");
    // moLotNo 上限 100 位
    assert!(check_lot(&"L".repeat(100)).is_ok());
    assert!(check_lot(&"L".repeat(101)).is_err());
  }

  #[test]
  fn station_codes_are_normalized_and_capped_at_50() {
    // 不传 / 空白 → None（统计全部工步）
    assert_eq!(check_station_codes(None).unwrap(), None);
    assert_eq!(check_station_codes(Some("   ")).unwrap(), None);
    assert_eq!(check_station_codes(Some(",,")).unwrap(), None);
    // 归一化：去空格、去空段，用英文逗号重连
    assert_eq!(check_station_codes(Some(" B10 , B20 ")).unwrap().as_deref(), Some("B10,B20"));
    assert_eq!(check_station_codes(Some("B10")).unwrap().as_deref(), Some("B10"));
    // 50 个通过、51 个拒绝
    let ok: String = (1..=50).map(|i| format!("B{i},")).collect();
    let too_many: String = (1..=51).map(|i| format!("B{i},")).collect();
    assert!(check_station_codes(Some(&ok)).is_ok());
    assert!(check_station_codes(Some(&too_many)).is_err());
  }

  #[test]
  fn root_trims_trailing_slash() {
    assert_eq!(root("http://host/frontApi/prod/"), "http://host/frontApi/prod");
    assert_eq!(root(" http://host/frontApi/prod "), "http://host/frontApi/prod");
  }

  /// 会话级端到端：`result: null` + `success: true` 必须走 `Ok(None)`，不得报错（标准 12.2）
  #[tokio::test]
  async fn session_serial_no_info_maps_null_result_to_none() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
      loop {
        let Ok((mut sock, _)) = listener.accept().await else { break };
        let mut buf = vec![0u8; 4096];
        let n = sock.read(&mut buf).await.unwrap_or(0);
        let head = String::from_utf8_lossy(&buf[..n]).to_string();
        let body = if head.contains("/api/Account/CreateToken") {
          r#"{"result":"mock-token","success":true,"unAuthorizedRequest":false,"error":null}"#
        } else {
          // 条码不存在：success 为 true 且 result 为 null
          r#"{"result":null,"success":true,"unAuthorizedRequest":false,"error":null}"#
        };
        let resp = format!(
          "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
          body.len()
        );
        let _ = sock.write_all(resp.as_bytes()).await;
        let _ = sock.shutdown().await;
      }
    });

    let cfg = Cfg {
      sysUrl: format!("http://{addr}"),
      userName: "u@example.com".into(),
      passWord: "p".into(),
      ..Default::default()
    };
    let session = Session::new(cfg);
    let found = session
      .serial_no_info(&format!("http://{addr}/frontApi/prod"), "SN-NOT-EXIST", None, None)
      .await
      .expect("result 为 null 不应报错");
    assert_eq!(found, None, "条码不存在应返回 None");
  }
}
