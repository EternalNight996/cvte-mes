use super::{header, Cfg, ErrorResultData, Response};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::*;
/// 工位信息请求结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct InsertAtSpiReq<'a> {
  /// 批次号
  pub moLotNo: &'a str,
  /// 条码
  pub snCode: &'a str,
  /// 版面
  /// - BOT = 1
  /// - TOP = 2
  /// - 卧式 = 3
  /// - 立式 = 4
  /// - BOT&TOP = 5
  pub surface: SurfaceType,
  /// 轨道
  /// - 公共轨 = 0
  /// - 1轨 = 1
  /// - 2轨 = 2
  pub rail: RailType,
}

/// 版面类型枚举
#[derive(Debug, Deserialize, Serialize, FromRepr, Clone)]
#[serde(from = "i32", into = "i32")]
#[repr(i32)]
pub enum SurfaceType {
  BOT = 1,
  TOP = 2,
  Horizontal = 3, // 卧式
  Vertical = 4,   // 立式
  BOTTOP = 5,     // BOT&TOP
}

/// 轨道类型枚举
#[derive(Debug, Deserialize, Serialize, FromRepr, Clone)]
#[serde(from = "i32", into = "i32")]
#[repr(i32)]
pub enum RailType {
  Public = 0, // 公共轨
  First = 1,  // 1轨
  Second = 2, // 2轨
}
// 实现 From trait 用于序列化和反序列化
impl From<i32> for RailType {
  fn from(value: i32) -> Self {
    Self::from_repr(value).unwrap_or(Self::Public)
  }
}

impl From<RailType> for i32 {
  fn from(value: RailType) -> i32 {
    value as i32
  }
}

impl From<i32> for SurfaceType {
  fn from(value: i32) -> Self {
    Self::from_repr(value).unwrap_or(Self::BOT)
  }
}

impl From<SurfaceType> for i32 {
  fn from(value: SurfaceType) -> i32 {
    value as i32
  }
}
/// # 序列扫描过站
/// ```
/// 请求头key
/// 测试:cmes_test_info
/// uat:cmes_uat_info
/// 正式:"cmes_"+工厂短码+"_info"
/// baseUrl地址
/// 测试:http://cmes-test.prod.gz.cvte.cn
/// uat:https://cmesqxauatprod.cvte.com
/// 正式:"http://" + 工厂短码+ "-cmesprod.gz.cvte.cn"
/// ```
///
/// //请求错误
/// {
///    "result": null,
///    "targetUrl": null,
///    "success": false,
///    "error": {
///      "code": 0,
///      "message": "请登录后操作！",
///      "details": null,
///      "validationErrors": null
///    },
///    "unAuthorizedRequest": true,
///    "__abp": true
///  }
///  //返回错误结果
///  {
///    "result": {
///      "result": "NG",
///      "message": "线别编号【string】不存在"
///    },
///    "targetUrl": null,
///    "success": true,
///    "error": null,
///    "unAuthorizedRequest": false,
///    "__abp": true
///  }
///  //返回成功结果
///  {
///    "result": {
///      "result": "PASS",
///      "message": ""
///    },
///    "targetUrl": null,
///    "success": true,
///    "error": null,
///    "unAuthorizedRequest": false,
///    "__abp": true
///  }
///   ```
pub async fn insert_at_spi<'a>(
  cfg: &Cfg,
  token: &str,
  req: &InsertAtSpiReq<'a>,
) -> e_utils::Result<String> {
  let url = format!("{}/api/services/feed/FeedSnRecord/InsertAtSpi", cfg.prodUrl);
  let headers = HeaderMap::from_iter([
    header(&cfg.tokenName, token)?,
    header("Content-Type", "application/json")?,
  ]);
  let response = reqwest::Client::new()
    .get(url)
    .headers(headers)
    .json(&json!( {
      "stationCode": &cfg.stationCode,
      "lineCode": &cfg.workCenter,
      "moLotNo": &req.moLotNo,
      "snCode": &req.snCode,
      "surface": &req.surface,
      "rail": &req.rail,
    }))
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json::<Response<ErrorResultData>>()
    .await
    .map_err(|e| e.to_string())?;

  // 根据success字段判断结果
  if response.result.is_success {
    Ok(response.result.data.to_string())
  } else {
    Err(response.result.to_str().unwrap_or_default().into())
  }
}
