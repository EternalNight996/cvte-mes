use super::{header, Cfg, Response};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::*;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RecordResult {
  pub result: String,
  pub message: String,
}
impl RecordResult {
  pub fn success(&self) -> bool {
    self.result == "PASS"
  }
  pub fn to_str(&self) -> e_utils::AnyResult<String> {
    Ok(serde_json::to_string_pretty(self)?)
  }
}
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

// 使用宏定义枚举
define_enum! {
  SurfaceType,
  BOT = 1,
  TOP = 2,
  Horizontal = 3,  // 卧式
  Vertical = 4,    // 立式
  BOTTOP = 5       // BOT&TOP
}

define_enum! {
  RailType,
  Public = 0,  // 公共轨
  First = 1,   // 1轨
  Second = 2   // 2轨
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
pub async fn insert_at_spi<'a>(cfg: &Cfg, token: &str, req: &InsertAtSpiReq<'a>) -> e_utils::AnyResult<String> {
  let url = format!("{}/api/services/feed/FeedSnRecord/InsertAtSpi", cfg.prodUrl);
  let headers = HeaderMap::from_iter([header(&cfg.tokenName, token)?, header("Content-Type", "application/json")?]);
  let response = reqwest::Client::new()
    .post(url)
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
    .await?
    .json::<Response<Option<RecordResult>>>()
    .await?;
  match (response.success, response.result) {
    (true, Some(res)) if res.success() => Ok(res.result),
    (true, Some(res)) => Err(res.to_str()?.into()),
    _ => Err(response.error.unwrap_or_default().to_str()?.into()),
  }
}
