//! 同步（阻塞）调用封装
//!
//! cvte-mes 的核心接口均为 async（tokio）；本模块用临时 tokio runtime 包装，
//! 供不便引入 async 运行时的调用方（如 egui 桌面端）直接同步调用。
//! 启用方式：`cargo build --features blocking` 或依赖方开启 `cvte-mes/blocking`。
use e_utils::AnyResult;

use crate::mes::*;

fn rt() -> tokio::runtime::Runtime {
  tokio::runtime::Runtime::new().expect("tokio runtime")
}

// ---------- CMES ----------

/// 登录获取 token（阻塞）
pub fn cmes_login(cfg: &cmes::Cfg) -> AnyResult<String> {
  rt().block_on(cmes::login::login(cfg))
}

/// SN 校验过站（CheckLotSnCode，阻塞）
pub fn cmes_check_sn_ok(cfg: &cmes::Cfg, token: &str, sn: &str) -> AnyResult<String> {
  rt().block_on(cmes::test_scan::check_lot_sn_code(cfg, token, sn))
}

/// 根据 SN 获取批次信息（阻塞）
pub fn cmes_get_info_by_sn(
  cfg: &cmes::Cfg,
  token: &str,
  sn: &str,
) -> AnyResult<cmes::test_scan::GetInfoBySnRes> {
  rt().block_on(cmes::test_scan::get_info_by_sn(cfg, token, sn))
}

/// 保存测试结果（报工，阻塞）
pub fn cmes_save_produce_test<'a>(
  cfg: &cmes::Cfg,
  token: &str,
  req: &cmes::test_scan::SaveProduceTestByAutoTestReq<'a>,
) -> AnyResult<String> {
  rt().block_on(cmes::test_scan::save_produce_test_by_auto_test(cfg, token, req))
}

/// 高复用：按 SN 获取批次号（登录→GetInfoBySn，阻塞）
pub fn cmes_mo_lot_no(cfg: &cmes::Cfg, sn: &str) -> AnyResult<String> {
  rt().block_on(cmes::mo_lot_no(cfg, sn))
}

/// 高复用：MES 报工（登录→SN校验→保存结果，阻塞）
pub fn cmes_report<'a>(
  cfg: &cmes::Cfg,
  req: &cmes::test_scan::SaveProduceTestByAutoTestReq<'a>,
) -> AnyResult<String> {
  rt().block_on(cmes::report(cfg, req))
}

/// 序列扫描过站（阻塞）
pub fn cmes_insert_at_spi<'a>(
  cfg: &cmes::Cfg,
  token: &str,
  req: cmes::record::InsertAtSpiReq<'a>,
) -> AnyResult<String> {
  rt().block_on(cmes::record::insert_at_spi(cfg, token, req))
}

/// CMES 会话（阻塞）：Token 缓存 + 失效自动重取并重试一次，适合 GUI 反复调用
///
/// 比一次性函数省：同一会话内多次业务调用只登录一次（默认有效 2 小时、到期前 5 分钟自动刷新）。
/// ```ignore
/// let s = CmesSession::new(&cfg)?;
/// let rows = s.get_other_list_by_sn("http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod", "SN001")?;
/// let done = s.get_sn_pack_finish_qty("http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod", "LOT001")?;
/// ```
pub struct CmesSession {
  rt: tokio::runtime::Runtime,
  inner: cmes::session::Session,
}

impl std::fmt::Debug for CmesSession {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CmesSession").field("session", &self.inner).finish_non_exhaustive()
  }
}

impl CmesSession {
  /// 新建会话（复用同一 tokio runtime 与 Token 缓存）
  pub fn new(cfg: &cmes::Cfg) -> AnyResult<Self> {
    Ok(Self {
      rt: rt(),
      inner: cmes::session::Session::new(cfg.clone()),
    })
  }
  /// 取 Token（缓存有效直接返回，否则登录并缓存）
  pub fn token(&self) -> AnyResult<String> {
    self.rt.block_on(self.inner.token())
  }
  /// 按产品条码获取组件关联列表（GetOtherListBySn）
  pub fn get_other_list_by_sn(&self, front_url: &str, sn_code: &str) -> AnyResult<Vec<cmes::pack::PackOtherItem>> {
    self.rt.block_on(self.inner.get_other_list_by_sn(front_url, sn_code))
  }
  /// 按批次获取 SN 包装完工数（GetSnPackFinishQty；服务端按批次限流 10 秒 30 次）
  pub fn get_sn_pack_finish_qty(&self, front_url: &str, mo_lot_no: &str) -> AnyResult<Vec<cmes::pack::SnPackFinishQty>> {
    self.rt.block_on(self.inner.get_sn_pack_finish_qty(front_url, mo_lot_no))
  }
  /// 按条码取批次投料单物料信息（GetFeedSheetByContainerName；服务端按条码限流 10 秒 30 次）
  pub fn feed_sheet_by_container_name(
    &self,
    front_url: &str,
    container_name: &str,
  ) -> AnyResult<Vec<cmes::bas::FeedSheetPart>> {
    self.rt.block_on(self.inner.feed_sheet_by_container_name(front_url, container_name))
  }
  /// 按批次取各工步过站数量（GetStationPassQty；`station_codes` 逗号分隔、最多 50 个，传 None 统计全部工步）
  pub fn station_pass_qty(
    &self,
    front_url: &str,
    mo_lot_no: &str,
    station_codes: Option<&str>,
  ) -> AnyResult<Vec<cmes::bas::StationPassRow>> {
    self.rt.block_on(self.inner.station_pass_qty(front_url, mo_lot_no, station_codes))
  }
  /// 按条码取条码详细信息（GetInfoBySn；**条码不存在返回 `Ok(None)`，不是异常**）
  pub fn serial_no_info(
    &self,
    front_url: &str,
    sn: &str,
    is_include_order: Option<bool>,
    component_basis_code: Option<&str>,
  ) -> AnyResult<Option<cmes::bas::SerialNoInfo>> {
    self
      .rt
      .block_on(self.inner.serial_no_info(front_url, sn, is_include_order, component_basis_code))
  }
}

// ---------- MAC（SN→MAC 绑定查询）----------

/// 查询 SN 绑定的 MAC 列表（阻塞；token 为 Some 走 CMES，None 走 ItAPI apikey）
pub fn mac_query_by_sn(
  cfg: &mac::MacCfg,
  token: Option<&str>,
  sn: &str,
) -> AnyResult<Vec<mac::MacBinding>> {
  rt().block_on(mac::query_macs_by_sn(cfg, token, sn))
}

// ---------- SCM ----------

/// 查询SCM工单信息（阻塞）
pub fn scm_mo_factory(
  cfg: &scm::wip::Cfg,
  req: &scm::wip::MoFactoryReq,
) -> AnyResult<Vec<scm::wip::MoInfo>> {
  rt().block_on(scm::wip::query_mo_factory(cfg, req))
}

/// 查询SCM投料单信息（阻塞）
pub fn scm_req_common(
  cfg: &scm::wip::Cfg,
  req: &scm::wip::FeedReq,
) -> AnyResult<Vec<scm::wip::FeedInfo>> {
  rt().block_on(scm::wip::query_req_common(cfg, req))
}

// ---------- ItAPI ----------

/// 批量查询 CPLM 生产文件（阻塞）
pub fn itapi_batch_production_documents(
  cfg: &itapi::Cfg,
  reqs: &[itapi::ProdDocReq],
) -> AnyResult<itapi::ProdDocResp> {
  rt().block_on(itapi::batch_production_documents(cfg, reqs))
}

/// 查询软件信息表生产文件列表（阻塞）
pub fn itapi_soft_info_list_v2(
  cfg: &itapi::Cfg,
  req: &itapi::SoftInfoReq,
) -> AnyResult<itapi::SoftInfoResp> {
  rt().block_on(itapi::soft_info_list_v2(cfg, req))
}

/// 通用 POST JSON（可选 apikey 鉴权头），返回响应 JSON（ItAPI 类接口联调/报公用）
pub fn http_post_json(
  url: &str,
  apikey: &str,
  body: &serde_json::Value,
) -> AnyResult<serde_json::Value> {
  rt().block_on(async {
    let mut builder = crate::mes::cmes::client()
      .post(url)
      .header("Content-Type", "application/json");
    if !apikey.is_empty() {
      builder = builder.header("apikey", apikey);
    }
    let text = builder
      .json(body)
      .send()
      .await
      .map_err(|e| format!("HTTP POST 网络异常: {url} ({e})"))?
      .json::<serde_json::Value>()
      .await
      .map_err(|e| format!("HTTP POST 响应解析失败: {e}"))?;
    Ok(text)
  })
}

/// CMES 报工：保存防混料 MAC 校验结果（登录 → SaveProduceTestByAutoTest）
/// `testItem` 为 MAC 名称列表（如 "WIFI MAC,LAN MAC"），`keyInfo` 为 {name,value} JSON
pub fn cmes_report_mac_result(
  cfg: &cmes::Cfg,
  sn: &str,
  is_pass: bool,
  macs: &[mac::MacBinding],
  worker_no: &str,
) -> AnyResult<String> {
  rt().block_on(async {
    let token = cmes::login::login(cfg).await?;
    let now = e_utils::chrono::Local::now()
      .format("%Y-%m-%d %H:%M:%S")
      .to_string();
    let macs_json: Vec<serde_json::Value> = macs
      .iter()
      .map(|m| serde_json::json!({ "name": m.name, "value": m.value }))
      .collect();
    let key_info = serde_json::to_string(&macs_json)?;
    let test_item = macs
      .iter()
      .map(|m| m.name.as_str())
      .collect::<Vec<_>>()
      .join(",");
    let req = cmes::test_scan::SaveProduceTestByAutoTestReq {
      lotSnCode: sn,
      isPass: is_pass,
      testTime: &now,
      workerNo: worker_no,
      keyInfo: &key_info,
      autoTestVersion: "cvte-mes",
      testStartDate: &now,
      testEndDate: &now,
      testItem: &test_item,
    };
    cmes::test_scan::save_produce_test_by_auto_test(cfg, &token, &req).await
  })
}
