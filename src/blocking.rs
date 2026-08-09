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

/// 序列扫描过站（阻塞）
pub fn cmes_insert_at_spi<'a>(
  cfg: &cmes::Cfg,
  token: &str,
  req: cmes::record::InsertAtSpiReq<'a>,
) -> AnyResult<String> {
  rt().block_on(cmes::record::insert_at_spi(cfg, token, req))
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
) -> AnyResult<serde_json::Value> {
  rt().block_on(itapi::batch_production_documents(cfg, reqs))
}

/// 查询软件信息表生产文件列表（阻塞）
pub fn itapi_soft_info_list_v2(
  cfg: &itapi::Cfg,
  req: &itapi::SoftInfoReq,
) -> AnyResult<serde_json::Value> {
  rt().block_on(itapi::soft_info_list_v2(cfg, req))
}

/// 通用 POST JSON（可选 apikey 鉴权头），返回响应 JSON（ItAPI 类接口联调/报公用）
pub fn http_post_json(
  url: &str,
  apikey: &str,
  body: &serde_json::Value,
) -> AnyResult<serde_json::Value> {
  rt().block_on(async {
    let mut builder = reqwest::Client::new()
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
