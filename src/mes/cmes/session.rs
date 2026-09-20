#![allow(non_snake_case)]
//! CMES 会话层：《CMES 对外接口对接标准》的通用约定实现
//!
//! - **Token 缓存**：默认有效期 2 小时，到期前 5 分钟主动刷新；并发调用只登录一次（文档 6）
//! - **失效重试**：HTTP 401 或 `unAuthorizedRequest === true` → 清除旧 Token、重新获取、**最多重试原请求一次**
//! - **响应分层**（文档 4.3 / 5.2）：
//!
//! | 顺序 | 判定 | 处理 |
//! |---|---|---|
//! | 1 | HTTP 500 | 读服务端提示（`error.message` / `error.details` / 响应体文本），**不解析 `result`** |
//! | 2 | HTTP 401 或 `unAuthorizedRequest` | Token 失效 → 重取后最多重试一次 |
//! | 3 | HTTP 200 且 `success !== true` | 业务失败，读 `error`，不使用 `result` |
//! | 4 | HTTP 200 且 `success === true` | 取 `result` |
//! | 5 | 其他 HTTP 状态码 | 未约定的接口异常 |
//! | 6 | 响应体非 JSON | 协议异常，保留脱敏后的原始响应 |
//!
//! 鉴权头按标准为 `x-auth-token`；现场前端 API 常用 `Cfg::tokenName`，非空时以配置为准，留空则回退标准名。
//!
//! ```ignore
//! use cvte_mes::mes::cmes::{session::Session, Cfg};
//! let s = Session::new(Cfg::default());
//! let rows = s.get_other_list_by_sn("http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod", "SN001").await?;
//! ```
use std::time::{Duration, SystemTime};

use serde_json::Value;
use tokio::sync::Mutex;

use super::{header, Cfg};

/// JWT Token 默认有效期（对接标准第 6 节：默认 2 小时）
pub const DEFAULT_TTL_SECS: u64 = 2 * 60 * 60;
/// 到期前提前刷新的秒数（抵消时钟误差与调用耗时）
pub const DEFAULT_REFRESH_AHEAD_SECS: u64 = 300;

/// 缓存的 Token 与到期时刻
#[derive(Debug, Clone)]
struct Cached {
  value: String,
  expires_at: SystemTime,
}

/// 响应判定结果（对接标准 4.3 / 5.2）
#[derive(Debug)]
enum Outcome {
  /// HTTP 200 且 `success === true`：值为 `result`
  Ok(Value),
  /// Token 失效：HTTP 401 或 `unAuthorizedRequest === true`
  Unauthorized(String),
  /// 其他失败：已提取可直接展示的服务端提示
  Failed(String),
}

/// CMES 会话：Token 缓存 + 失效自动重取（对接标准 6）
#[derive(Debug)]
pub struct Session {
  cfg: Cfg,
  ttl: Duration,
  refresh_ahead: Duration,
  token: Mutex<Option<Cached>>,
}

impl Session {
  /// 默认配置：有效期 2 小时、到期前 5 分钟刷新
  pub fn new(cfg: Cfg) -> Self {
    Self::with_ttl(cfg, DEFAULT_TTL_SECS, DEFAULT_REFRESH_AHEAD_SECS)
  }
  /// 自定义有效期与提前刷新量（秒）
  pub fn with_ttl(cfg: Cfg, ttl_secs: u64, refresh_ahead_secs: u64) -> Self {
    Self {
      cfg,
      ttl: Duration::from_secs(ttl_secs),
      refresh_ahead: Duration::from_secs(refresh_ahead_secs),
      token: Mutex::new(None),
    }
  }
  /// 配置引用
  pub fn cfg(&self) -> &Cfg {
    &self.cfg
  }
  /// 取 Token：缓存仍有效则直接返回，否则登录并缓存（并发调用只登录一次）
  pub async fn token(&self) -> e_utils::AnyResult<String> {
    let mut guard = self.token.lock().await;
    if let Some(cached) = guard.as_ref() {
      if SystemTime::now() + self.refresh_ahead < cached.expires_at {
        return Ok(cached.value.clone());
      }
    }
    let value = super::login::login(&self.cfg).await?;
    *guard = Some(self.cache(value.clone()));
    Ok(value)
  }
  /// 强制刷新 Token（收到 401 / `unAuthorizedRequest` 时调用；失败即停止重试，由调用方告警）
  pub async fn refresh(&self) -> e_utils::AnyResult<String> {
    let mut guard = self.token.lock().await;
    *guard = None;
    let value = super::login::login(&self.cfg).await?;
    *guard = Some(self.cache(value.clone()));
    Ok(value)
  }
  /// 按《对接标准》发起 GET 并解析：自动带鉴权头；Token 失效时重取后最多重试一次
  pub async fn get_json(&self, url: &str, query: &[(&str, &str)]) -> e_utils::AnyResult<Value> {
    let client = super::client();
    let mut retried = false;
    loop {
      let token = self.token().await?;
      let request = client.get(url).headers(auth_headers(&self.cfg, &token)?).query(query);
      let (status, body) = read_body(request).await?;
      match classify(status, &body) {
        Outcome::Ok(value) => return Ok(value),
        Outcome::Unauthorized(msg) => {
          if retried {
            return Err(format!("CMES：Token 失效且重试一次后仍未通过鉴权（{msg}），请检查账号、权限与服务状态").into());
          }
          retried = true;
          self.refresh().await?;
        }
        Outcome::Failed(msg) => return Err(msg.into()),
      }
    }
  }
  fn cache(&self, value: String) -> Cached {
    Cached {
      value,
      expires_at: SystemTime::now() + self.ttl,
    }
  }
}

/// 已有 Token 的一次性 GET（不做 Token 重试）：沿用同一套响应分层，供显式传 Token 的接口复用
pub async fn get_json_with_token(cfg: &Cfg, token: &str, url: &str, query: &[(&str, &str)]) -> e_utils::AnyResult<Value> {
  let request = super::client().get(url).headers(auth_headers(cfg, token)?).query(query);
  let (status, body) = read_body(request).await?;
  match classify(status, &body) {
    Outcome::Ok(value) => Ok(value),
    Outcome::Unauthorized(msg) => Err(format!("CMES：Token 已失效（{msg}），请重新登录获取 Token 后重试").into()),
    Outcome::Failed(msg) => Err(msg.into()),
  }
}

/// 鉴权请求头（标准字段 `x-auth-token`；`Cfg::tokenName` 非空时以现场配置为准）
pub fn auth_headers(cfg: &Cfg, token: &str) -> e_utils::AnyResult<reqwest::header::HeaderMap> {
  Ok(reqwest::header::HeaderMap::from_iter([header(auth_name(cfg), token)?]))
}

/// 鉴权头名：配置为空时回退对接标准名 `x-auth-token`
fn auth_name(cfg: &Cfg) -> &str {
  let name = cfg.tokenName.trim();
  if name.is_empty() {
    "x-auth-token"
  } else {
    name
  }
}

/// 发送请求并取回「状态码 + 原始响应文本」（保留原文，非 JSON 也能给出协议异常）
async fn read_body(request: reqwest::RequestBuilder) -> e_utils::AnyResult<(u16, String)> {
  let response = request
    .send()
    .await
    .map_err(|e| format!("CMES：无法链接服务端，请检查网络状态! ({e})"))?;
  let status = response.status().as_u16();
  let body = response.text().await.map_err(|e| format!("CMES：读取响应失败 ({e})"))?;
  Ok((status, body))
}

/// 按对接标准 4.3 / 5.2 分层判定响应
fn classify(status: u16, body: &str) -> Outcome {
  match status {
    // 500：服务端异常或服务端友好提示，不解析 result
    500 => Outcome::Failed(server_hint(body).unwrap_or_else(|| "CMES 服务端异常（HTTP 500）".into())),
    // 401：Token 失效，重试由调用方负责
    401 => Outcome::Unauthorized(server_hint(body).unwrap_or_else(|| "未授权（HTTP 401）".into())),
    200 => match serde_json::from_str::<Value>(body) {
      Err(e) => Outcome::Failed(format!("CMES 响应不是合法 JSON（协议异常）：{e}；原始响应：{}", clip(body.trim(), 300))),
      Ok(value) => {
        if value.get("unAuthorizedRequest").and_then(Value::as_bool).unwrap_or(false) {
          return Outcome::Unauthorized(server_hint(body).unwrap_or_else(|| "未授权".into()));
        }
        if value.get("success").and_then(Value::as_bool).unwrap_or(false) {
          Outcome::Ok(value.get("result").cloned().unwrap_or(Value::Null))
        } else {
          Outcome::Failed(server_hint(body).unwrap_or_else(|| "CMES 业务处理失败（success 非 true）".into()))
        }
      }
    },
    other => Outcome::Failed(format!("CMES 未约定的 HTTP 状态码 {other}（按接口异常处理，请记录后联系接口维护人员）")),
  }
}

/// 提取服务端提示：`error.message` → `error.details` → 顶层 `msg` / `message` → 纯文本响应体
fn server_hint(body: &str) -> Option<String> {
  let text = body.trim();
  if text.is_empty() {
    return None;
  }
  if let Ok(value) = serde_json::from_str::<Value>(text) {
    let candidates = [
      value.pointer("/error/message"),
      value.pointer("/error/details"),
      value.get("msg"),
      value.get("message"),
    ];
    // 逐个跳过空串：error.message 为空时继续看 error.details
    let hint = candidates
      .into_iter()
      .flatten()
      .filter_map(Value::as_str)
      .map(str::trim)
      .find(|s| !s.is_empty());
    if let Some(s) = hint {
      return Some(clip(s, 300));
    }
  }
  Some(clip(text, 300))
}

/// 截断过长文本：错误提示不刷屏（凭据脱敏由调用方负责，勿在此拼接 Token / 密码摘要）
fn clip(text: &str, max: usize) -> String {
  if text.chars().count() <= max {
    text.to_string()
  } else {
    format!("{}…", text.chars().take(max).collect::<String>())
  }
}

#[cfg(test)]
mod tests {
  use std::sync::atomic::{AtomicUsize, Ordering};
  use std::sync::Arc;

  use tokio::io::{AsyncReadExt, AsyncWriteExt};

  use super::*;

  #[test]
  fn classify_reads_result_on_success() {
    let body = r#"{"result":[{"id":"1"}],"success":true,"unAuthorizedRequest":false,"__abp":true,"error":null}"#;
    match classify(200, body) {
      Outcome::Ok(v) => assert_eq!(v[0]["id"], "1"),
      other => panic!("应为成功：{other:?}"),
    }
  }

  #[test]
  fn classify_business_failure_prefers_error_message() {
    let body = r#"{"result":null,"success":false,"error":{"code":0,"message":"用户名或密码错误","details":null}}"#;
    match classify(200, body) {
      Outcome::Failed(msg) => assert_eq!(msg, "用户名或密码错误"),
      other => panic!("应为业务失败：{other:?}"),
    }
  }

  #[test]
  fn classify_treats_unauthorized_flag_as_unauthorized() {
    let body = r#"{"result":null,"success":false,"unAuthorizedRequest":true,"error":{"message":"请登录后操作！"}}"#;
    match classify(200, body) {
      Outcome::Unauthorized(msg) => assert_eq!(msg, "请登录后操作！"),
      other => panic!("应为未授权：{other:?}"),
    }
  }

  #[test]
  fn classify_500_reads_hint_and_ignores_result() {
    let body = r#"{"result":{"fake":"不应被解析"},"success":false,"error":{"message":"产线【001】不存在","details":"workCenter"}}"#;
    match classify(500, body) {
      Outcome::Failed(msg) => assert_eq!(msg, "产线【001】不存在"),
      other => panic!("应为失败：{other:?}"),
    }
  }

  #[test]
  fn classify_non_json_body_is_protocol_error() {
    match classify(200, "<html>502 Bad Gateway</html>") {
      Outcome::Failed(msg) => assert!(msg.contains("协议异常"), "{msg}"),
      other => panic!("应为协议异常：{other:?}"),
    }
  }

  #[test]
  fn classify_unknown_status_is_uncontracted_error() {
    match classify(502, "{}") {
      Outcome::Failed(msg) => assert!(msg.contains("502"), "{msg}"),
      other => panic!("应为未约定状态：{other:?}"),
    }
  }

  #[test]
  fn server_hint_falls_back_to_details_then_plain_text() {
    assert_eq!(
      server_hint(r#"{"error":{"message":"","details":"details 兜底"}}"#).as_deref(),
      Some("details 兜底")
    );
    assert_eq!(server_hint("  系统维护中  ").as_deref(), Some("系统维护中"));
    assert_eq!(server_hint("   ").as_deref(), None);
  }

  #[test]
  fn auth_name_falls_back_to_standard_header() {
    let cfg = Cfg::default();
    assert_eq!(auth_name(&cfg), "x-auth-token");
    let cfg = Cfg {
      tokenName: "cmes_test_info".into(),
      ..Default::default()
    };
    assert_eq!(auth_name(&cfg), "cmes_test_info");
  }

  /// 最小 HTTP mock：登录路径返回 token，业务路径按预设队列应答
  /// 返回（base_url, 登录次数, 业务调用次数）
  async fn spawn_mock(business: Vec<(u16, &'static str)>) -> (String, Arc<AtomicUsize>, Arc<AtomicUsize>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let logins = Arc::new(AtomicUsize::new(0));
    let calls = Arc::new(AtomicUsize::new(0));
    let (login_hits, call_hits) = (logins.clone(), calls.clone());
    tokio::spawn(async move {
      let mut queue = business.into_iter();
      loop {
        let Ok((mut sock, _)) = listener.accept().await else { break };
        let mut buf = vec![0u8; 8192];
        let Ok(n) = sock.read(&mut buf).await else { continue };
        let head = String::from_utf8_lossy(&buf[..n]).to_string();
        let (status, body) = if head.contains("/api/Account/CreateToken") {
          login_hits.fetch_add(1, Ordering::SeqCst);
          (200u16, r#"{"result":"mock-token","success":true,"unAuthorizedRequest":false,"__abp":true,"error":null}"#)
        } else {
          call_hits.fetch_add(1, Ordering::SeqCst);
          queue.next().unwrap_or((200, r#"{"result":null,"success":true}"#))
        };
        let reason = match status {
          200 => "OK",
          401 => "Unauthorized",
          500 => "Internal Server Error",
          _ => "Status",
        };
        let response = format!("HTTP/1.1 {status} {reason}
Content-Type: application/json
Content-Length: {}
Connection: close

{body}", body.len());
        let _ = sock.write_all(response.as_bytes()).await;
        let _ = sock.shutdown().await;
      }
    });
    (format!("http://{addr}"), logins, calls)
  }

  fn cfg_for(base: &str) -> Cfg {
    Cfg {
      sysUrl: base.to_string(),
      userName: "u@example.com".into(),
      passWord: "p".into(),
      ..Default::default()
    }
  }

  #[tokio::test]
  async fn session_retries_once_after_401_then_succeeds() {
    let (base, logins, calls) = spawn_mock(vec![
      (401, r#"{"result":null,"success":false,"unAuthorizedRequest":true,"error":{"message":"请登录后操作！"}}"#),
      (200, r#"{"result":[{"snCode":"SN1"}],"success":true,"unAuthorizedRequest":false,"__abp":true,"error":null}"#),
    ])
    .await;
    let session = Session::new(cfg_for(&base));
    let value = session
      .get_json(&format!("{base}/api/services/pack/Pack/GetOtherListBySn"), &[("snCode", "SN1")])
      .await
      .unwrap();
    assert_eq!(value[0]["snCode"], "SN1");
    assert_eq!(calls.load(Ordering::SeqCst), 2, "401 后应重试原请求一次");
    assert_eq!(logins.load(Ordering::SeqCst), 2, "重试前应重新获取 Token");
  }

  #[tokio::test]
  async fn session_gives_up_after_one_retry() {
    let (base, _logins, calls) = spawn_mock(vec![
      (401, r#"{"success":false,"unAuthorizedRequest":true,"error":{"message":"请登录后操作！"}}"#),
      (401, r#"{"success":false,"unAuthorizedRequest":true,"error":{"message":"仍未授权"}}"#),
    ])
    .await;
    let session = Session::new(cfg_for(&base));
    let err = session
      .get_json(&format!("{base}/api/services/pack/Pack/GetOtherListBySn"), &[("snCode", "SN1")])
      .await
      .unwrap_err()
      .to_string();
    assert!(err.contains("重试一次后仍未通过鉴权"), "{err}");
    assert_eq!(calls.load(Ordering::SeqCst), 2, "只允许重试一次，不得循环重试");
  }

  #[tokio::test]
  async fn session_does_not_retry_on_http_500() {
    let (base, logins, calls) = spawn_mock(vec![(
      500,
      r#"{"result":{"fake":"不应被解析"},"success":false,"error":{"message":"产线【001】不存在"}}"#,
    )])
    .await;
    let session = Session::new(cfg_for(&base));
    let err = session
      .get_json(&format!("{base}/api/services/pack/Pack/GetOtherListBySn"), &[("snCode", "SN1")])
      .await
      .unwrap_err()
      .to_string();
    assert_eq!(err, "产线【001】不存在");
    assert_eq!(calls.load(Ordering::SeqCst), 1, "500 不触发 Token 重试");
    assert_eq!(logins.load(Ordering::SeqCst), 1);
  }

  #[tokio::test]
  async fn session_reuses_cached_token_across_calls() {
    let (base, logins, calls) = spawn_mock(vec![
      (200, r#"{"result":[],"success":true,"unAuthorizedRequest":false,"__abp":true,"error":null}"#),
      (200, r#"{"result":[],"success":true,"unAuthorizedRequest":false,"__abp":true,"error":null}"#),
    ])
    .await;
    let session = Session::new(cfg_for(&base));
    let url = format!("{base}/api/services/pack/Pack/GetSnPackFinishQty");
    session.get_json(&url, &[("moLotNo", "LOT001")]).await.unwrap();
    session.get_json(&url, &[("moLotNo", "LOT001")]).await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(logins.load(Ordering::SeqCst), 1, "缓存有效期内只登录一次");
  }
}
