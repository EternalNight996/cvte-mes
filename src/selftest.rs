//! 内置自检：用本地 mock 服务端验证会话层机制，**不访问生产环境、不产生业务副作用**
//!
//! 覆盖《CMES 对外接口对接标准》的机制性约定（这些场景脚本/生产环境无法自然构造）：
//!
//! | # | 检查 | 对应约定 |
//! |---|---|---|
//! | 1 | 连续取 Token 只登录一次且值相同 | 文档 6 · Token 缓存 |
//! | 2 | 4 个并发取 Token 仍只登录一次（mock 侧交叉计数为 1） | 文档 6 · 并发只登录一次 |
//! | 3 | HTTP 401 → 自动重取并重试原请求一次 → 成功 | 文档 4.3 / 5.2 |
//! | 4 | 持续 401 时只刷新一次即报错（重试有上限，不无限循环） | 文档 5.2 |
//! | 5 | 显式 Token 路径把 401 识别为「Token 已失效」 | 文档 4.3 |
//! | 6 | HTTP 500 读取服务端提示（不解析 result） | 文档 4.3 顺序 1 |
//! | 7 | HTTP 200 且 success=false → 用 error.message | 文档 4.3 顺序 3 |
//! | 8 | HTTP 200 但非 JSON → 协议异常 | 文档 4.3 顺序 6 |
//! | 9 | 正常 200 成功取 result | 文档 4.3 顺序 4 |
//! | 10 | blocking 同步路径可用（需 blocking feature） | GUI 场景 |
//! | 11 | 第 10 章：按条码取投料单物料行解析正确（含 bool/小数） | V1.1 文档 10.3 |
//! | 12 | 第 11 章：各工步过站行解析正确，`stationCode` 为 `null` 亦不误判 | V1.1 文档 11.4 |
//! | 13 | 第 12 章：条码详细信息对象解析正确（含 `isIncludeOrder=false` 的空值） | V1.1 文档 12.3 |
//! | 14 | 第 12 章：**条码不存在（`result` 为 `null`）返回 `Ok(None)` 而不报错** | V1.1 文档 12.2 |
//! | 15 | 前置校验：空白 `containerName` 直接报参数错误，不发起请求 | V1.1 文档 10.4 |
//!
//! CLI：`--api CMES --task SelfTest:Session`
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::mes::cmes::{session, session::Session, Cfg};

/// 本地 mock 服务端句柄
#[derive(Debug)]
pub struct Mock {
  /// 形如 `http://127.0.0.1:PORT`
  pub base: String,
  /// mock 侧累计收到的登录请求数（用于交叉验证「只登录一次」）
  pub login_hits: Arc<AtomicUsize>,
}

/// 按 mock 地址构造一份可用的 Cfg
pub fn mock_cfg(mock: &Mock) -> Cfg {
  Cfg {
    stationName: "selftest".into(),
    prodUrl: mock.base.clone(),
    sysUrl: mock.base.clone(),
    userName: "selftest@mock".into(),
    passWord: "selftest".into(),
    tokenName: "x-auth-token".into(),
    causeNo: "SELFTEST".into(),
    workCenter: "SELF".into(),
    testFrameidNo: "SELF".into(),
    stationCode: "SELF".into(),
  }
}

/// 启动本地 mock 服务端（随机端口，仅监听 127.0.0.1）
pub async fn start_mock() -> std::io::Result<Mock> {
  let listener = TcpListener::bind("127.0.0.1:0").await?;
  let addr = listener.local_addr()?;
  let login_hits = Arc::new(AtomicUsize::new(0));
  let once_hits = Arc::new(AtomicUsize::new(0));
  let always_hits = Arc::new(AtomicUsize::new(0));

  let login_srv = login_hits.clone();
  tokio::spawn(async move {
    loop {
      let Ok((mut sock, _)) = listener.accept().await else {
        break;
      };
      let login_hits = login_srv.clone();
      let once_hits = once_hits.clone();
      let always_hits = always_hits.clone();
      tokio::spawn(async move {
        let mut buf = vec![0u8; 4096];
        let n = sock.read(&mut buf).await.unwrap_or(0);
        let req = String::from_utf8_lossy(&buf[..n]).to_string();
        let path = req.split_whitespace().nth(1).unwrap_or("/").to_string();
        let (code, body) = route(&path, &login_hits, &once_hits, &always_hits);
        let resp = format!(
          "HTTP/1.1 {code}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
          body.len()
        );
        let _ = sock.write_all(resp.as_bytes()).await;
        let _ = sock.shutdown().await;
      });
    }
  });

  Ok(Mock {
    base: format!("http://{addr}"),
    login_hits,
  })
}

fn route(
  path: &str,
  logins: &AtomicUsize,
  once: &AtomicUsize,
  always: &AtomicUsize,
) -> (&'static str, String) {
  let p = path.split('?').next().unwrap_or("/");
  match p {
    // 登录端点：每次登录返回不同的 Token，便于识别「到底登了几次」
    "/api/Account/CreateToken" => {
      let n = logins.fetch_add(1, Ordering::SeqCst) + 1;
      (
        "200 OK",
        json!({ "result": format!("mock-token-{n}"), "success": true, "unAuthorizedRequest": false })
          .to_string(),
      )
    }
    // 第一次 401，之后 200：验证「重取后重试一次」
    "/mock/once401" => {
      let n = once.fetch_add(1, Ordering::SeqCst) + 1;
      if n == 1 {
        ("401 Unauthorized", unauthorized_body())
      } else {
        ("200 OK", json!({ "result": { "ok": true }, "success": true }).to_string())
      }
    }
    // 永远 401：验证重试上限
    "/mock/always401" => {
      always.fetch_add(1, Ordering::SeqCst);
      ("401 Unauthorized", unauthorized_body())
    }
    "/mock/ok" => ("200 OK", json!({ "result": [1, 2], "success": true }).to_string()),
    // 业务失败：HTTP 200 但 success=false
    "/mock/bizfail" => (
      "200 OK",
      json!({ "result": null, "success": false, "error": { "code": 400, "message": "条码不存在" } })
        .to_string(),
    ),
    // HTTP 500：应读取服务端提示
    "/mock/http500" => (
      "500 Internal Server Error",
      json!({ "result": null, "success": false, "error": { "code": 500, "message": "服务器内部错误" } })
        .to_string(),
    ),
    // 非 JSON：协议异常
    "/mock/notjson" => ("200 OK", "<html>not json</html>".to_string()),

    // ---------- 《对接标准 V1.1》第 10~12 章新增接口：按文档真实报文应答 ----------
    // 10：投料单物料行（应发数 > 0，按物料编码倒序）
    "/api/services/bas/FeedSheetOrder/GetFeedSheetByContainerName" => (
      "200 OK",
      json!({
        "result": [
          { "partId": 10001, "partNo": "006.xxx", "partName": "物料描述", "needNum": 12.0,
            "quantityPerAssembly": 1.0, "isLotControl": true, "moLotNo": "LOT001" },
          { "partId": 10002, "partNo": "005.yyy", "partName": "包装箱", "needNum": 1.0,
            "quantityPerAssembly": 0.1, "isLotControl": false, "moLotNo": "LOT001" }
        ],
        "success": true, "unAuthorizedRequest": false, "error": null
      })
      .to_string(),
    ),
    // 11：各工步过站行（含 stationCode 为 null 的一行，验证 Option 容错）
    "/api/services/bas/ScanHistory/GetStationPassQty" => (
      "200 OK",
      json!({
        "result": [
          { "moLotNo": "LOT001", "stationId": "8f2b", "stationCode": "B10", "stationName": "组装",
            "sortNo": 10, "moQty": 200, "type": "整机", "passQty": 120 },
          { "moLotNo": "LOT001", "stationId": "8f2b", "stationCode": "B10", "stationName": "组装",
            "sortNo": 10, "moQty": 200, "type": "部件", "passQty": 40 },
          { "moLotNo": "LOT001", "stationId": "9a3c", "stationCode": null, "stationName": "测试",
            "sortNo": 20, "moQty": 200, "type": "整机", "passQty": 118 }
        ],
        "success": true, "unAuthorizedRequest": false, "error": null
      })
      .to_string(),
    ),
    // 12：条码详细信息；sn=NOT-EXIST 时按标准 12.2 返回 success=true 且 result=null
    "/api/services/bas/SerialNoCache/GetInfoBySn" => {
      if query_param(path, "sn") == "NOT-EXIST" {
        (
          "200 OK",
          json!({ "result": null, "success": true, "unAuthorizedRequest": false, "error": null }).to_string(),
        )
      } else {
        (
          "200 OK",
          json!({
            "result": {
              "id": "3fa85f64", "orderId": "5b21c0de", "orderNo": "PO20261001001", "moLotNo": "LOT001",
              "orderQty": 200.0, "qty": 200, "componentBasisId": "9c2d5e1a",
              "componentBasisCode": "CMP_FACTORY", "sn": "MOCK-SN", "snCode": "MOCK-SN",
              "reelId": "REEL001", "snNum": 1, "snQty": 1.0, "innerNum": 4.0,
              "subType": "组件类型编码", "partId": 10001, "partNo": "P-0001", "partName": "物料描述",
              "productAbName": "产品简称", "productModel": "产品型号A", "sellModel": "销售型号A",
              "electricParam": "电气参数", "isPuzzle": false,
              "serialNoRuleSetId": "rule-1", "serialNoHeadId": "head-1"
            },
            "success": true, "unAuthorizedRequest": false, "error": null
          })
          .to_string(),
        )
      }
    }
    _ => (
      "404 Not Found",
      json!({ "success": false, "error": { "code": 404, "message": "not found" } }).to_string(),
    ),
  }
}

/// 取查询串里某个参数的值（mock 内部用；自检的取值不含需要百分号解码的字符）
fn query_param(path: &str, key: &str) -> String {
  let query = path.split_once('?').map(|(_, q)| q).unwrap_or("");
  query
    .split('&')
    .filter_map(|kv| kv.split_once('='))
    .find(|(k, _)| *k == key)
    .map(|(_, v)| v.to_string())
    .unwrap_or_default()
}

fn unauthorized_body() -> String {
  json!({
    "result": null,
    "success": false,
    "unAuthorizedRequest": true,
    "error": { "code": 401, "message": "登录已过期" }
  })
  .to_string()
}

/// 单项检查结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct Check {
  pub name: String,
  pub pass: bool,
  pub detail: String,
}

fn ok(name: &str, detail: impl Into<String>) -> Check {
  Check {
    name: name.into(),
    pass: true,
    detail: detail.into(),
  }
}

fn bad(name: &str, detail: impl Into<String>) -> Check {
  Check {
    name: name.into(),
    pass: false,
    detail: detail.into(),
  }
}

/// 运行全部自检：返回 (是否全部通过, 明细 JSON)
pub async fn run_all() -> (bool, Value) {
  let checks = match start_mock().await {
    Ok(mock) => {
      let mut v = run_session_checks(&mock).await;
      v.push(run_blocking_check(&mock).await);
      v
    }
    Err(e) => vec![bad("mock_server", format!("无法启动本地 mock 服务端: {e}"))],
  };
  let passed = checks.iter().filter(|c| c.pass).count();
  let total = checks.len();
  let all_pass = passed == total;
  let value = json!({ "all_pass": all_pass, "passed": passed, "total": total, "checks": checks });
  (all_pass, value)
}

async fn run_session_checks(mock: &Mock) -> Vec<Check> {
  let mut out = Vec::new();
  let cfg = mock_cfg(mock);

  // 1. 连续两次取 Token：值相同，且程序内计数与 mock 侧计数都为 1
  let s = Session::new(cfg.clone());
  let before = mock.login_hits.load(Ordering::SeqCst);
  let r1 = s.token().await;
  let r2 = s.token().await;
  let hits = mock.login_hits.load(Ordering::SeqCst) - before;
  match (r1, r2) {
    (Ok(t1), Ok(t2)) if t1 == t2 && s.login_count() == 1 && hits == 1 => out.push(ok(
      "token_cache_single_login",
      format!("两次取回同一 Token，程序内登录 {} 次 / mock 收到 {hits} 次", s.login_count()),
    )),
    (Ok(t1), Ok(t2)) => out.push(bad(
      "token_cache_single_login",
      format!(
        "值相同={} 程序内登录={} mock 收到={}（期望均 1 次）",
        t1 == t2,
        s.login_count(),
        hits
      ),
    )),
    (a, b) => out.push(bad(
      "token_cache_single_login",
      format!("取 Token 失败: {:?} / {:?}", a.err(), b.err()),
    )),
  }

  // 2. 四个并发取 Token：仍只登录一次
  let s = Session::new(cfg.clone());
  let before = mock.login_hits.load(Ordering::SeqCst);
  let (a, b, c, d) = tokio::join!(s.token(), s.token(), s.token(), s.token());
  let hits = mock.login_hits.load(Ordering::SeqCst) - before;
  let vals: Vec<String> = [a, b, c, d].into_iter().filter_map(Result::ok).collect();
  if vals.len() == 4 && vals.iter().all(|v| *v == vals[0]) && s.login_count() == 1 && hits == 1 {
    out.push(ok(
      "token_cache_concurrent_single_login",
      format!("4 个并发调用返回同一 Token，程序内登录 1 次 / mock 收到 {hits} 次"),
    ));
  } else {
    out.push(bad(
      "token_cache_concurrent_single_login",
      format!(
        "成功 {} 个，程序内登录 {} 次，mock 收到 {hits} 次（期望 4/1/1）",
        vals.len(),
        s.login_count()
      ),
    ));
  }

  // 3. 401 → 自动重取并重试一次 → 成功
  let s = Session::new(cfg.clone());
  s.seed_token("stale-token".into(), 3600).await;
  match s.get_json(&format!("{}/mock/once401", mock.base), &[]).await {
    Ok(v) if s.login_count() == 1 => out.push(ok(
      "token_retry_after_401",
      format!("401 后自动重取并重试成功（刷新登录 {} 次），返回 {v}", s.login_count()),
    )),
    Ok(_) => out.push(bad(
      "token_retry_after_401",
      format!("重试成功但刷新登录 {} 次（期望 1）", s.login_count()),
    )),
    Err(e) => out.push(bad("token_retry_after_401", format!("未按预期重试成功: {e}"))),
  }

  // 4. 持续 401：只刷新一次即报错（不无限重试）
  let s = Session::new(cfg.clone());
  s.seed_token("stale-token".into(), 3600).await;
  match s.get_json(&format!("{}/mock/always401", mock.base), &[]).await {
    Err(e) if s.login_count() == 1 => out.push(ok(
      "token_retry_only_once",
      format!("持续 401 只刷新一次即报错（有重试上限）：{e}"),
    )),
    Err(e) => out.push(bad(
      "token_retry_only_once",
      format!("登录 {} 次（期望 1，说明重试有上限）：{e}", s.login_count()),
    )),
    Ok(_) => out.push(bad("token_retry_only_once", "持续 401 却返回成功")),
  }

  // 5. 显式 Token 路径的失效识别
  match session::get_json_with_token(&cfg, "invalid-token", &format!("{}/mock/always401", mock.base), &[]).await
  {
    Err(e) if e.to_string().contains("已失效") => out.push(ok(
      "token_invalid_detected",
      format!("识别为 Token 已失效：{e}"),
    )),
    Err(e) => out.push(bad("token_invalid_detected", format!("报错但未提示失效: {e}"))),
    Ok(_) => out.push(bad("token_invalid_detected", "失效 Token 却返回成功")),
  }

  // 6. HTTP 500：读取服务端提示
  match session::get_json_with_token(&cfg, "t", &format!("{}/mock/http500", mock.base), &[]).await {
    Err(e) if e.to_string().contains("服务器内部错误") => out.push(ok(
      "http500_server_message",
      format!("HTTP 500 提取到服务端提示：{e}"),
    )),
    Err(e) => out.push(bad("http500_server_message", format!("HTTP 500 提示未提取到: {e}"))),
    Ok(_) => out.push(bad("http500_server_message", "HTTP 500 却返回成功")),
  }

  // 7. 业务失败：HTTP 200 + success=false → error.message
  match session::get_json_with_token(&cfg, "t", &format!("{}/mock/bizfail", mock.base), &[]).await {
    Err(e) if e.to_string().contains("条码不存在") => out.push(ok(
      "biz_fail_message",
      format!("业务失败提取 error.message：{e}"),
    )),
    Err(e) => out.push(bad("biz_fail_message", format!("业务失败提示不符: {e}"))),
    Ok(_) => out.push(bad("biz_fail_message", "业务失败却返回成功")),
  }

  // 8. 非 JSON 响应 → 协议异常
  match session::get_json_with_token(&cfg, "t", &format!("{}/mock/notjson", mock.base), &[]).await {
    Err(e) => {
      let s = e.to_string();
      if s.to_lowercase().contains("json") || s.contains("协议") {
        out.push(ok("non_json_protocol_error", format!("非 JSON 识别为协议异常：{s}")));
      } else {
        out.push(bad("non_json_protocol_error", format!("报错信息不含协议提示: {s}")));
      }
    }
    Ok(_) => out.push(bad("non_json_protocol_error", "非 JSON 却解析成功")),
  }

  // 9. 正常 200 成功取 result
  match session::get_json_with_token(&cfg, "t", &format!("{}/mock/ok", mock.base), &[]).await {
    Ok(v) => out.push(ok("http200_result", format!("正常响应取到 result：{v}"))),
    Err(e) => out.push(bad("http200_result", format!("正常响应却失败: {e}"))),
  }

  // ---------- 《对接标准 V1.1》第 10~12 章新增接口：按文档真实报文验证解析 ----------

  // 10. 第 10 章：投料单物料行
  let s = Session::new(cfg.clone());
  match s.feed_sheet_by_container_name(&mock.base, "MOCK-SN").await {
    Ok(rows) if rows.len() == 2 => {
      let first = &rows[0];
      if first.partNo == "006.xxx" && first.needNum == 12.0 && first.quantityPerAssembly == 1.0
        && first.isLotControl && !rows[1].isLotControl
      {
        out.push(ok(
          "bas_feed_sheet_parts",
          format!("第 10 章解析正确：{} 行，首行 {} 应发 {} 单位用量 {}", rows.len(), first.partNo, first.needNum, first.quantityPerAssembly),
        ));
      } else {
        out.push(bad(
          "bas_feed_sheet_parts",
          format!("字段不符：partNo={} needNum={} isLotControl={}", first.partNo, first.needNum, first.isLotControl),
        ));
      }
    }
    Ok(rows) => out.push(bad("bas_feed_sheet_parts", format!("行数不符（期望 2）：{}", rows.len()))),
    Err(e) => out.push(bad("bas_feed_sheet_parts", format!("请求失败: {e}"))),
  }

  // 11. 第 11 章：各工步过站行（第三行 stationCode 为 null）
  match s.station_pass_qty(&mock.base, "LOT001", Some("B10, B20")).await {
    Ok(rows) if rows.len() == 3 => {
      if rows[0].r#type == "整机" && rows[0].passQty == 120 && rows[0].moQty == 200
        && rows[1].r#type == "部件"
        && rows[2].stationCode.is_none()
      {
        out.push(ok(
          "bas_station_pass_qty",
          format!("第 11 章解析正确：{} 行，stationCode 为 null 的行未误判", rows.len()),
        ));
      } else {
        out.push(bad(
          "bas_station_pass_qty",
          format!("字段不符：type={} passQty={} 第三行 stationCode={:?}", rows[0].r#type, rows[0].passQty, rows[2].stationCode),
        ));
      }
    }
    Ok(rows) => out.push(bad("bas_station_pass_qty", format!("行数不符（期望 3）：{}", rows.len()))),
    Err(e) => out.push(bad("bas_station_pass_qty", format!("请求失败: {e}"))),
  }

  // 12. 第 12 章：条码详细信息对象
  match s.serial_no_info(&mock.base, "MOCK-SN", None, None).await {
    Ok(Some(info)) => {
      if info.sn == "MOCK-SN" && info.sn == info.snCode && info.orderQty == 200.0
        && info.partId == Some(10001) && !info.isPuzzle
      {
        out.push(ok(
          "bas_serial_no_info_object",
          format!("第 12 章解析正确：sn={} 批次 {} 单据 {}", info.sn, info.moLotNo, info.orderNo),
        ));
      } else {
        out.push(bad(
          "bas_serial_no_info_object",
          format!("字段不符：sn={} snCode={} orderQty={} partId={:?}", info.sn, info.snCode, info.orderQty, info.partId),
        ));
      }
    }
    Ok(None) => out.push(bad("bas_serial_no_info_object", "条码存在却返回 None")),
    Err(e) => out.push(bad("bas_serial_no_info_object", format!("请求失败: {e}"))),
  }

  // 13. 第 12 章核心约定：条码不存在（success=true 且 result=null）→ Ok(None)，不是异常
  match s.serial_no_info(&mock.base, "NOT-EXIST", None, None).await {
    Ok(None) => out.push(ok(
      "bas_serial_no_info_not_found_is_none",
      "条码不存在返回 Ok(None)（result 为 null 且 success 为 true，未按异常处理）",
    )),
    Ok(Some(info)) => out.push(bad(
      "bas_serial_no_info_not_found_is_none",
      format!("条码不存在却解析出对象：sn={}", info.sn),
    )),
    Err(e) => out.push(bad(
      "bas_serial_no_info_not_found_is_none",
      format!("条码不存在被当成异常（违反标准 12.2）：{e}"),
    )),
  }

  // 14. 前置校验：空白 containerName 直接报参数错误，不发起请求
  match s.feed_sheet_by_container_name(&mock.base, "   ").await {
    Err(e) if e.to_string().contains("参数错误") => out.push(ok(
      "bas_blank_param_rejected",
      format!("空参数前置拒绝且未发起请求：{e}"),
    )),
    Err(e) => out.push(bad("bas_blank_param_rejected", format!("报错但未提示参数错误: {e}"))),
    Ok(rows) => out.push(bad(
      "bas_blank_param_rejected",
      format!("空参数竟返回 {} 行（应前置拒绝）", rows.len()),
    )),
  }

  out
}

async fn run_blocking_check(mock: &Mock) -> Check {
  #[cfg(feature = "blocking")]
  {
    let cfg = mock_cfg(mock);
    let base = mock.base.clone();
    let joined = tokio::task::spawn_blocking(move || {
      // 跨线程要求 Send：错误就地转成 String（AnyResult 的错误类型不是 Send）
      let login = crate::blocking::cmes_login(&cfg).map_err(|e| e.to_string());
      let body = json!({ "probe": 1 });
      let post =
        crate::blocking::http_post_json(&format!("{base}/mock/ok"), "", &body).map_err(|e| e.to_string());
      (login, post)
    })
    .await;
    match joined {
      Ok((Ok(token), Ok(_))) if token.starts_with("mock-token-") => ok(
        "blocking_sync_path",
        format!("blocking 同步路径可用（登录拿到 {token}）"),
      ),
      Ok((login, post)) => bad(
        "blocking_sync_path",
        format!("blocking 调用异常: login={:?} post={:?}", login.err(), post.err()),
      ),
      Err(e) => bad("blocking_sync_path", format!("blocking 任务 join 失败: {e}")),
    }
  }
  #[cfg(not(feature = "blocking"))]
  {
    let _ = mock;
    bad(
      "blocking_sync_path",
      "未启用 blocking feature（构建加 --features input-api,blocking 才会执行此项）",
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// 自检即回归测试：会话层机制被改动后，`cargo test` 会立刻发现
  #[tokio::test]
  async fn selftest_checks_pass() {
    let (all_pass, detail) = run_all().await;
    if cfg!(feature = "blocking") {
      assert!(all_pass, "自检未全部通过: {detail}");
      return;
    }
    // 未启用 blocking feature 时，只允许 blocking 一项失败
    let empty = Vec::new();
    let checks = detail["checks"].as_array().unwrap_or(&empty);
    let failed: Vec<&str> = checks
      .iter()
      .filter(|c| c["pass"] == serde_json::Value::Bool(false))
      .filter_map(|c| c["name"].as_str())
      .collect();
    assert!(
      failed.is_empty() || failed == vec!["blocking_sync_path"],
      "自检失败项: {failed:?} / 明细: {detail}"
    );
  }
}
