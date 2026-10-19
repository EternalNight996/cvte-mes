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

// ==================== GetStationPassQty 闭环验收 ====================

/// 单条闭环校验结果
#[derive(Debug, Clone, Default, Serialize)]
pub struct VerifyCheck {
  pub name: String,
  pub pass: bool,
  pub detail: String,
}

fn chk(name: &str, pass: bool, detail: impl Into<String>) -> VerifyCheck {
  VerifyCheck {
    name: name.to_string(),
    pass,
    detail: detail.into(),
  }
}

/// [`verify_station_pass_qty`] 的验收结果
///
/// 验收口径（对接标准 11.1 / 11.2）：
/// 1. **不传** `stationCodes` → 必须返回该批次的**全部工步**数据
/// 2. **传** `stationCodes` → 只返回**指定工步**的数据，且与全量结果逐行自洽
#[derive(Debug, Clone, Default, Serialize)]
pub struct StationPassQtyVerify {
  pub moLotNo: String,
  /// `PASS` 两侧自洽；`FAIL` 不自洽；`NO_DATA` 该批次无过站数据（或被口径全部过滤），无法闭环
  pub verdict: String,
  pub all_rows: usize,
  /// 全量查询里出现的工步编码（去重升序）
  pub all_station_codes: Vec<String>,
  /// 本次用于「指定工步」复查的工步编码
  pub checked_station_codes: Vec<String>,
  pub filtered_rows: usize,
  pub checks: Vec<VerifyCheck>,
}

/// `GetStationPassQty` 闭环验收：**全量查询 → 指定工步复查 → 两侧逐项对比**
///
/// - `max_stations`：从全量结果中挑多少个工步做「指定工步」复查；传 0 按 3 处理
/// - 会发起 **两次** 业务请求（服务端按 `moLotNo` 限流 10 秒 30 次，勿在循环里高频调用）
///
/// 覆盖的检查项：
/// - 全量查询（不传 `stationCodes`）必须返回该批次的全部工步数据
/// - 指定工步查询只返回被指定的工步
/// - 每一行（工步, 类型, 过站数）都与全量结果一致
/// - 指定工步的行数 == 全量结果中对应工步的行数
/// - 只复查部分工步时，结果行数必须收窄（证明确实在过滤）；复查全部时须等于全量
pub async fn verify_station_pass_qty(
  cfg: &Cfg,
  token: &str,
  front_url: &str,
  mo_lot_no: &str,
  max_stations: usize,
) -> e_utils::AnyResult<StationPassQtyVerify> {
  let lot = check_lot(mo_lot_no)?.to_string();
  let take = if max_stations == 0 { 3 } else { max_stations };

  // 第 1 次：**不传** stationCodes —— 验收「反馈该批次所有工站的数据」
  let all = station_pass_qty(cfg, token, front_url, &lot, None).await?;

  let mut all_codes: Vec<String> = all
    .iter()
    .filter_map(|r| r.stationCode.as_deref())
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .map(str::to_string)
    .collect();
  all_codes.sort();
  all_codes.dedup();

  let mut out = StationPassQtyVerify {
    moLotNo: lot.clone(),
    all_rows: all.len(),
    all_station_codes: all_codes.clone(),
    ..Default::default()
  };

  // 该批次无过站数据：闭环无法成立，明确报 NO_DATA，而不是假装通过
  if all.is_empty() {
    out.verdict = "NO_DATA".into();
    out.checks.push(chk(
      "all_stations_data",
      false,
      "全量查询（不传 stationCodes）返回空数组：该批次无符合口径的过站记录，无法完成闭环验收。\
       请换一个正在生产/有过站数据的批次；若确认该批次应有记录，则需排查是否被「重置（重投/返工）不计入」\
       或「条码类型仅限工厂条码与高温条码」的口径过滤",
    ));
    return Ok(out);
  }

  out.checks.push(chk(
    "all_stations_data",
    true,
    format!(
      "全量查询返回 {} 行、{} 个工步：{}",
      all.len(),
      all_codes.len(),
      all_codes.join(",")
    ),
  ));

  // ---- 以下按标准 11.1 / 11.4 校验「全量结果」自身的口径（一次响应即可判定）----

  // D1：每行的 moLotNo 必须就是请求的批次
  let wrong_lot: Vec<String> = all
    .iter()
    .filter(|r| r.moLotNo.trim() != lot)
    .map(|r| r.moLotNo.clone())
    .collect();
  out.checks.push(chk(
    "lot_matches_requested",
    wrong_lot.is_empty(),
    if wrong_lot.is_empty() {
      format!("全部 {} 行的 moLotNo 均为 {}", all.len(), lot)
    } else {
      format!("出现与请求批次 {} 不符的行：{}", lot, wrong_lot.join(","))
    },
  ));

  // D2：moQty 是「批次数量（计划数）」，每行都应携带且一致
  let mut mo_qty_values: Vec<i64> = all.iter().map(|r| r.moQty).collect();
  mo_qty_values.sort_unstable();
  mo_qty_values.dedup();
  out.checks.push(chk(
    "mo_qty_consistent",
    mo_qty_values.len() == 1 && mo_qty_values[0] > 0,
    if mo_qty_values.len() == 1 {
      format!("每行 moQty 均为 {}（批次计划数）", mo_qty_values[0])
    } else {
      format!("moQty 不一致或非正值：{mo_qty_values:?}")
    },
  ));

  // D3：结果按 sortNo 升序（文档 11.4「结果按此升序」）
  let bad_sort: Vec<String> = all
    .windows(2)
    .filter(|w| w[0].sortNo > w[1].sortNo)
    .map(|w| format!("{}->{}", w[0].sortNo, w[1].sortNo))
    .collect();
  out.checks.push(chk(
    "sorted_by_sort_no",
    bad_sort.is_empty(),
    if bad_sort.is_empty() {
      "结果按 sortNo 升序".to_string()
    } else {
      format!("出现逆序（前>后）：{}", bad_sort.join(","))
    },
  ));

  // D4：同一工步内「整机在前、部件在后」（文档 11.4）
  //     以连续同 stationCode 的区段为单位；区段内一旦出现「部件」后又出现「整机」即违反
  let mut order_violations: Vec<String> = Vec::new();
  let mut seen_part = false;
  for i in 0..all.len() {
    let cur_part = all[i].r#type == "部件";
    let prev_same_station = i > 0
      && all[i - 1].stationCode.as_deref().map(str::trim) == all[i].stationCode.as_deref().map(str::trim);
    if !prev_same_station {
      seen_part = false; // 进入新工步区段，重置
    }
    if cur_part {
      seen_part = true;
    } else if seen_part && all[i].r#type == "整机" {
      order_violations.push(format!(
        "{}：部件之后又出现整机",
        all[i].stationCode.as_deref().unwrap_or("<null>")
      ));
    }
  }
  out.checks.push(chk(
    "machine_before_part",
    order_violations.is_empty(),
    if order_violations.is_empty() {
      "同一工步内整机行均在部件行之前".to_string()
    } else {
      order_violations.join("；")
    },
  ));

  // 行存在但全部没有工步编码：无法按工步复查
  if all_codes.is_empty() {
    out.checks.push(chk(
      "station_codes_present",
      false,
      "全量结果中所有行的 stationCode 均为 null，无法指定工步复查（服务端工步缓存未命中？）",
    ));
    out.verdict = "FAIL".into();
    return Ok(out);
  }

  if all_codes.len() == 1 {
    out.checks.push(chk(
      "multi_station_available",
      true,
      format!("该批次仅 {} 个工步（{}），无法体现「过滤收窄」，其余检查照常执行", all_codes.len(), all_codes[0]),
    ));
  }

  let checked: Vec<String> = all_codes.iter().take(take).cloned().collect();
  let codes_arg = checked.join(",");
  out.checked_station_codes = checked.clone();

  // 第 2 次：**传** stationCodes —— 验收「支持返回指定工站」
  let filtered = station_pass_qty(cfg, token, front_url, &lot, Some(&codes_arg)).await?;
  out.filtered_rows = filtered.len();

  // C1：指定工步查询不能为空（全量里明明有这些工步）
  out.checks.push(chk(
    "filtered_non_empty",
    !filtered.is_empty(),
    format!("指定工步 [{}] 查询返回 {} 行", codes_arg, filtered.len()),
  ));

  // C2：返回行的工步必须都在被指定的集合内
  let bad: Vec<String> = filtered
    .iter()
    .filter(|r| match r.stationCode.as_deref().map(str::trim) {
      Some(code) if !code.is_empty() => !checked.iter().any(|c| c == code),
      _ => true, // 指定工步查询却返回 stationCode 为空的行，同样不符
    })
    .map(|r| r.stationCode.clone().unwrap_or_else(|| "<null>".into()))
    .collect();
  out.checks.push(chk(
    "only_requested_stations",
    bad.is_empty(),
    if bad.is_empty() {
      format!("返回行的工步均属于指定集合 [{}]", codes_arg)
    } else {
      format!("出现未被指定工步的行：{}", bad.join(","))
    },
  ));

  // C3：每个（工步, 类型）的过站数必须与全量结果一致
  let mut mismatches: Vec<String> = Vec::new();
  for f in &filtered {
    let hit = all.iter().find(|a| {
      a.stationCode.as_deref().map(str::trim) == f.stationCode.as_deref().map(str::trim) && a.r#type == f.r#type
    });
    match hit {
      Some(a) if a.passQty == f.passQty && a.moLotNo == f.moLotNo => {}
      Some(a) => mismatches.push(format!(
        "{}/{}：指定查询 passQty={} 全量={}",
        f.stationCode.as_deref().unwrap_or("<null>"),
        f.r#type,
        f.passQty,
        a.passQty
      )),
      None => mismatches.push(format!(
        "{}/{}：全量结果中不存在该（工步,类型）组合",
        f.stationCode.as_deref().unwrap_or("<null>"),
        f.r#type
      )),
    }
  }
  out.checks.push(chk(
    "qty_matches_all",
    mismatches.is_empty(),
    if mismatches.is_empty() {
      format!("{} 行的（工步,类型,过站数）均与全量结果一致", filtered.len())
    } else {
      mismatches.join("；")
    },
  ));

  // C4：行数 == 全量结果中属于这些工步的行数
  let expected = all
    .iter()
    .filter(|a| match a.stationCode.as_deref().map(str::trim) {
      Some(code) => checked.iter().any(|c| c == code),
      None => false,
    })
    .count();
  out.checks.push(chk(
    "row_count_matches",
    filtered.len() == expected,
    format!("指定工步行数 {} / 全量中对应行数 {}", filtered.len(), expected),
  ));

  // C5：抽查子集时必须「收窄」。基准用**命名工步**的行数：stationCode 为 null 的行
  // （服务端工步缓存未命中）无法按工步指定，不参与收窄判断
  let named_total = all
    .iter()
    .filter(|a| a.stationCode.as_deref().map(str::trim).map(|s| !s.is_empty()).unwrap_or(false))
    .count();
  if checked.len() < all_codes.len() {
    out.checks.push(chk(
      "filter_reduces",
      filtered.len() < named_total,
      format!(
        "只复查 {}/{} 个命名工步：指定查询 {} 行 应 < 命名工步行数 {}",
        checked.len(),
        all_codes.len(),
        filtered.len(),
        named_total
      ),
    ));
  } else {
    out.checks.push(chk(
      "filter_reduces",
      true,
      format!(
        "已复查全部 {} 个命名工步，无法体现收窄；另有 {} 行 stationCode 为 null（不可按工步指定）",
        all_codes.len(),
        all.len().saturating_sub(named_total)
      ),
    ));
  }

  out.verdict = if out.checks.iter().all(|c| c.pass) { "PASS" } else { "FAIL" }.into();
  Ok(out)
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

  /// 闭环验收端到端：mock 按 stationCodes 真实过滤，验证「全部工站」与「指定工站」两侧自洽
  #[tokio::test]
  async fn verify_station_pass_qty_closes_the_loop() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    // 三条过站记录：B10 整机 120 / B10 部件 40 / B20 整机 118
    let rows = [
      r#"{"moLotNo":"LOT1","stationId":"s1","stationCode":"B10","stationName":"组装","sortNo":10,"moQty":200,"type":"整机","passQty":120}"#,
      r#"{"moLotNo":"LOT1","stationId":"s1","stationCode":"B10","stationName":"组装","sortNo":10,"moQty":200,"type":"部件","passQty":40}"#,
      r#"{"moLotNo":"LOT1","stationId":"s2","stationCode":"B20","stationName":"测试","sortNo":20,"moQty":200,"type":"整机","passQty":118}"#,
    ];
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
      loop {
        let Ok((mut sock, _)) = listener.accept().await else { break };
        let mut buf = vec![0u8; 8192];
        let n = sock.read(&mut buf).await.unwrap_or(0);
        let head = String::from_utf8_lossy(&buf[..n]).to_string();
        // 请求行形如 "GET /path?q=1 HTTP/1.1"，取中间那段当作 URL
        let req_url = head.lines().next().unwrap_or("").split_whitespace().nth(1).unwrap_or("").to_string();
        let body = if req_url.contains("/api/Account/CreateToken") {
          r#"{"result":"tok","success":true,"unAuthorizedRequest":false,"error":null}"#.to_string()
        } else {
          // 只从 URL 的查询串取 stationCodes（reqwest 会把逗号编码成 %2C）
          let wanted: Option<Vec<String>> = req_url.split_once('?').and_then(|(_, q)| {
            q.split('&')
              .filter_map(|kv| kv.split_once('='))
              .find(|(k, _)| *k == "stationCodes")
              .map(|(_, v)| v.replace("%2C", ","))
              .map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
          });
          let picked: Vec<&str> = rows
            .iter()
            .copied()
            .filter(|r| match &wanted {
              None => true, // 不传 -> 全部工步
              Some(codes) => codes.iter().any(|c| r.contains(&format!("\"stationCode\":\"{c}\""))),
            })
            .collect();
          format!(
            r#"{{"result":[{}],"success":true,"unAuthorizedRequest":false,"error":null}}"#,
            picked.join(",")
          )
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
    let front = format!("http://{addr}/frontApi/prod");

    // 只复查 1 个工步（B10）：应 PASS，且结果从 3 行收窄到 2 行
    let v = verify_station_pass_qty(&cfg, "tok", &front, "LOT1", 1).await.unwrap();
    assert_eq!(v.verdict, "PASS", "checks: {:?}", v.checks);
    assert_eq!(v.all_rows, 3, "全量查询应返回 3 行（全部工站）");
    assert_eq!(v.all_station_codes, vec!["B10".to_string(), "B20".to_string()]);
    assert_eq!(v.checked_station_codes, vec!["B10".to_string()]);
    assert_eq!(v.filtered_rows, 2, "只查 B10 应返回 2 行");
    assert!(v.checks.iter().all(|c| c.pass), "checks: {:?}", v.checks);

    // 复查全部工步：应 PASS，且指定查询 == 全量
    let v2 = verify_station_pass_qty(&cfg, "tok", &front, "LOT1", 9).await.unwrap();
    assert_eq!(v2.verdict, "PASS", "checks: {:?}", v2.checks);
    assert_eq!(v2.filtered_rows, 3);
  }

  /// 该批次无过站数据时必须报 NO_DATA（不假装 PASS，也不报错）
  #[tokio::test]
  async fn verify_station_pass_qty_reports_no_data() {
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
          r#"{"result":"tok","success":true}"#.to_string()
        } else {
          r#"{"result":[],"success":true}"#.to_string()
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
    let v = verify_station_pass_qty(&cfg, "tok", &format!("http://{addr}/frontApi/prod"), "LOT1", 3)
      .await
      .unwrap();
    assert_eq!(v.verdict, "NO_DATA");
    assert_eq!(v.all_rows, 0);
    assert!(v.checks.iter().any(|c| c.name == "all_stations_data" && !c.pass));
  }

  /// 闭环验收的判别力：服务端**无视** stationCodes（照样返回全部工站）时必须判 FAIL
  #[tokio::test]
  async fn verify_station_pass_qty_detects_ignored_filter() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let all_rows = r#"[
      {"moLotNo":"LOT1","stationId":"s1","stationCode":"B10","stationName":"组装","sortNo":10,"moQty":200,"type":"整机","passQty":120},
      {"moLotNo":"LOT1","stationId":"s1","stationCode":"B10","stationName":"组装","sortNo":10,"moQty":200,"type":"部件","passQty":40},
      {"moLotNo":"LOT1","stationId":"s2","stationCode":"B20","stationName":"测试","sortNo":20,"moQty":200,"type":"整机","passQty":118}
    ]"#;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
      loop {
        let Ok((mut sock, _)) = listener.accept().await else { break };
        let mut buf = vec![0u8; 8192];
        let n = sock.read(&mut buf).await.unwrap_or(0);
        let req_url = String::from_utf8_lossy(&buf[..n])
          .lines()
          .next()
          .unwrap_or("")
          .split_whitespace()
          .nth(1)
          .unwrap_or("")
          .to_string();
        // 关键：无论是否传 stationCodes，一律返回全部三行（模拟「过滤未生效」）
        let body = if req_url.contains("/api/Account/CreateToken") {
          r#"{"result":"tok","success":true}"#.to_string()
        } else {
          format!(r#"{{"result":{all_rows},"success":true}}"#)
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
    // 只抽查 1 个工站（B10），若服务端真过滤应只剩 2 行；返回 3 行即说明过滤没生效
    let v = verify_station_pass_qty(&cfg, "tok", &format!("http://{addr}/frontApi/prod"), "LOT1", 1)
      .await
      .unwrap();
    assert_eq!(v.verdict, "FAIL", "服务端无视 stationCodes 时必须判 FAIL：{:?}", v.checks);
    let failed: Vec<&str> = v.checks.iter().filter(|c| !c.pass).map(|c| c.name.as_str()).collect();
    assert!(failed.contains(&"only_requested_stations"), "应检出越界工站：{failed:?}");
    assert!(failed.contains(&"row_count_matches"), "应检出数量不符：{failed:?}");
    assert!(failed.contains(&"filter_reduces"), "应检出未收窄：{failed:?}");
  }

  /// 口径判别力：sortNo 逆序 / 同工步「部件」排在「整机」之前，都必须判 FAIL（标准 11.1 / 11.4）
  #[tokio::test]
  async fn verify_station_pass_qty_detects_bad_order() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    // 故意违规：sortNo 20 在前、10 在后；同一工步 B10 先「部件」后「整机」
    let rows = r#"[
      {"moLotNo":"LOT1","stationId":"s1","stationCode":"B10","stationName":"组装","sortNo":20,"moQty":200,"type":"部件","passQty":40},
      {"moLotNo":"LOT1","stationId":"s1","stationCode":"B10","stationName":"组装","sortNo":10,"moQty":200,"type":"整机","passQty":120}
    ]"#;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
      loop {
        let Ok((mut sock, _)) = listener.accept().await else { break };
        let mut buf = vec![0u8; 8192];
        let n = sock.read(&mut buf).await.unwrap_or(0);
        let url = String::from_utf8_lossy(&buf[..n])
          .lines()
          .next()
          .unwrap_or("")
          .split_whitespace()
          .nth(1)
          .unwrap_or("")
          .to_string();
        let body = if url.contains("/api/Account/CreateToken") {
          r#"{"result":"tok","success":true}"#.to_string()
        } else {
          format!(r#"{{"result":{rows},"success":true}}"#)
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
    let v = verify_station_pass_qty(&cfg, "tok", &format!("http://{addr}/frontApi/prod"), "LOT1", 1)
      .await
      .unwrap();
    assert_eq!(v.verdict, "FAIL", "排序/口径违规时必须判 FAIL：{:?}", v.checks);
    let failed: Vec<&str> = v.checks.iter().filter(|c| !c.pass).map(|c| c.name.as_str()).collect();
    assert!(failed.contains(&"sorted_by_sort_no"), "应检出 sortNo 逆序：{failed:?}");
    assert!(failed.contains(&"machine_before_part"), "应检出部件排在整机之前：{failed:?}");
  }
}
