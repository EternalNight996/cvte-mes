#![allow(non_snake_case)]
use std::str::FromStr as _;

pub use r#type::*;
use reqwest::header::{HeaderName, HeaderValue};
pub mod login;
pub mod record;
pub mod test_scan;
pub fn header(k: &str, v: &str) -> Result<(HeaderName, HeaderValue), Box<dyn std::error::Error>> {
  Ok((HeaderName::from_str(k)?, HeaderValue::from_str(v)?))
}
mod r#type {
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
    pub result: T,
    pub targetUrl: Option<String>,
    pub success: bool,
    pub error: Option<Error>,
    pub unAuthorizedRequest: bool,
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
    pub code: i32,
    pub message: String,
    pub details: Option<String>,
    pub validation_errors: Option<Vec<String>>,
  }
  impl Error {
    pub fn to_str(&self) -> e_utils::Result<String> {
      Ok(serde_json::to_string_pretty(self)?)
    }
  }
  /// 错误信息结构体
  #[derive(Deserialize, Serialize, Debug, Default)]
  pub struct ErrorResultData {
    pub isSuccess: bool,
    pub code: String,
    pub msg: String,
    #[serde(default)]
    pub data: Value,
  }
  impl ErrorResultData {
    pub fn to_str(&self) -> e_utils::Result<String> {
      Ok(serde_json::to_string_pretty(self)?)
    }
  }
}
