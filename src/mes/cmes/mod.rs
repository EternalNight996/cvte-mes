#![allow(non_snake_case)]
use std::str::FromStr as _;

use e_utils::AnyRes as _;
pub use r#type::*;
use reqwest::header::{HeaderName, HeaderValue};
use std::time::Duration;

/// 安装 rustls 加密后端（ring）
///
/// reqwest 启用的是 `rustls-no-provider`：不先安装 provider，构造 Client 时 reqwest 会直接
/// panic（`No rustls crypto provider is configured`），HTTP/HTTPS 一视同仁。故在建 Client 前幂等安装一次。
pub fn ensure_tls() {
  use std::sync::Once;
  static ONCE: Once = Once::new();
  ONCE.call_once(|| {
    // 已安装过时返回 Err（携带既有 provider），忽略即可
    let _ = rustls::crypto::ring::default_provider().install_default();
  });
}

/// 带超时的 HTTP 客户端（默认 10s，避免 MES 服务器不可达时卡死工位流程）
pub fn client() -> reqwest::Client {
  ensure_tls();
  // 对接标准 4.1：JSON 接口统一带 Accept；请求体为 JSON 时 reqwest 的 .json() 会自动补 Content-Type
  let mut default_headers = reqwest::header::HeaderMap::new();
  default_headers.insert(
    reqwest::header::ACCEPT,
    reqwest::header::HeaderValue::from_static("application/json"),
  );
  reqwest::Client::builder()
    .default_headers(default_headers)
    .timeout(Duration::from_secs(10))
    .build()
    .expect("构造 reqwest client 失败")
}
pub mod feed_sheet;
pub mod login;
pub mod pack;
pub mod record;
pub mod session;
pub mod test_scan;
pub fn header(k: &str, v: &str) -> e_utils::Result<(HeaderName, HeaderValue)> {
  Ok((HeaderName::from_str(k).any()?, HeaderValue::from_str(v).any()?))
}

/// 高复用：按 SN 获取批次号 moLotNo（登录 → GetInfoBySn；SN 支持 $ 分隔取左侧）
pub async fn mo_lot_no(cfg: &Cfg, sn: &str) -> e_utils::AnyResult<String> {
  let token = login::login(cfg).await?;
  let code = sn.split('$').next().unwrap_or(sn);
  let info = test_scan::get_info_by_sn(cfg, &token, code).await?;
  Ok(info.moLotNo)
}

/// 高复用：MES 报工（登录 → SN 校验过站 CheckLotSnCode → 保存结果 SaveProduceTestByAutoTest）
pub async fn report(cfg: &Cfg, req: &test_scan::SaveProduceTestByAutoTestReq<'_>) -> e_utils::AnyResult<String> {
  let token = login::login(cfg).await?;
  let code = req.lotSnCode.split('$').next().unwrap_or(req.lotSnCode);
  test_scan::check_lot_sn_code(cfg, &token, code).await?;
  test_scan::save_produce_test_by_auto_test(cfg, &token, req).await
}
mod r#type {
  use e_utils::AnyRes;
  use serde::{Deserialize, Serialize};
  use serde_json::Value;
  /// Mes信息
  #[derive(Deserialize, Debug, Clone, Serialize, Default)]
  pub struct Cfg {
    /// 站位名
    pub stationName: String,
    /// Prod域名
    pub prodUrl: String,
    /// Sys域名
    pub sysUrl: String,
    /// 用户名
    pub userName: String,
    /// 密码
    pub passWord: String,
    /// 令牌名
    pub tokenName: String,
    /// 公司标识
    pub causeNo: String,
    /// 产线编号
    pub workCenter: String,
    /// 测试架编号
    pub testFrameidNo: String,
    /// 测试工位 station_code
    pub stationCode: String,
  }

  /// 登录响应结构体
  #[derive(Deserialize, Debug)]
  pub struct Response<T: Default> {
    #[serde(default)]
    pub result: T,
    #[serde(default)]
    pub targetUrl: Option<String>,
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub error: Option<Error>,
    #[serde(default)]
    pub unAuthorizedRequest: bool,
    #[serde(default)]
    pub __abp: bool,
  }
  //   /// 结果类型枚举
  //   #[derive(Deserialize, Debug)]
  //   #[serde(untagged)] // 允许不同的结构体
  //   pub enum LoginResult {
  //     ErrorResult(ErrorResultData), // 异常结果
  //     SuccessResult(String),        // 成功结果
  //   }

  /// 错误信息结构体
  #[derive(Deserialize, Debug, Serialize, Default)]
  pub struct Error {
    #[serde(default)]
    pub code: i32,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub validation_errors: Option<Vec<String>>,
  }
  impl Error {
    pub fn to_str(&self) -> e_utils::Result<String> {
      serde_json::to_string_pretty(self).any()
    }
  }
  /// 错误信息结构体
  #[derive(Deserialize, Serialize, Debug, Default)]
  pub struct ErrorResultData {
    #[serde(default)]
    pub isSuccess: bool,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub msg: String,
    #[serde(default)]
    pub data: Value,
  }
  impl ErrorResultData {
    pub fn to_str(&self) -> e_utils::Result<String> {
      serde_json::to_string_pretty(self).any()
    }
  }
}
