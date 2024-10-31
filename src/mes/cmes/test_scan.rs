#![allow(non_snake_case)]

use reqwest::header::{self};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{header, Cfg, ErrorResultData, Response};

/// # 判断测试工步
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
/// #校验异常
/// ```
/// {
///   "result": {
///       "isSuccess": false,
///       "code": "400",
///       "msg": "最后扫描工步是（包装），当前工艺路线已过站完成"
///   },
///   "targetUrl": null,
///   "success": true,
///   "error": null,
///   "unAuthorizedRequest": false,
///   "__abp": true
/// }
/// //校验成功
/// {
///   "result": {
///       "isSuccess": true,
///       "code": "200",
///       "msg": ""
///   },
///   "targetUrl": null,
///   "success": true,
///   "error": null,
///   "unAuthorizedRequest": false,
///   "__abp": true
/// }
/// ```
pub async fn check_lot_sn_code(cfg: &Cfg, token: &str, lot_sn_code: &str) -> e_utils::AnyResult<String> {
  let url = format!("{}/api/services/test/TestScan/CheckLotSnCode", cfg.prodUrl);
  let headers = header::HeaderMap::from_iter([header(&cfg.tokenName, token)?, header("Content-Type", "application/json")?]);
  let response = reqwest::Client::new()
    .post(url)
    .headers(headers)
    .json(&json!( {
      "lotSnCode": lot_sn_code,
      "stationCode": &cfg.stationCode,
    }))
    .send()
    .await?
    .json::<Response<Option<ErrorResultData>>>()
    .await?;
  match (response.success, response.result, response.error) {
    (true, Some(res), _) if res.isSuccess => Ok(res.msg),
    (true, Some(res), _) => Err(res.to_str()?.into()),
    (_, _, Some(err)) => Err(err.to_str()?.into()),
    _ => Err("未知错误".into()),
  }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SaveProduceTestByAutoTestReq<'a> {
  /// 条码/SN code
  pub lotSnCode: &'a str,
  /// 是否通过/Pass or not
  pub isPass: bool,
  /// 测试上传时间/Test upload time
  pub testTime: &'a str, // 使用字符串表示日期时间
  /// 测试员
  pub workerNo: &'a str, // 测试员
  /// TV测试KEY信息/Test KEY information
  pub keyInfo: &'a str,
  /// TV测试版本/Test software version
  pub autoTestVersion: &'a str,
  /// 测试开始时间/Test start time
  pub testStartDate: &'a str, // 使用字符串表示日期时间
  /// 测试结束时间/Test end time
  pub testEndDate: &'a str, // 使用字符串表示日期时间
  /// 测试项名称/Test item name
  pub testItem: &'a str,
}
/// # 保存测试结果
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
/// #校验异常
/// ```
/// {
///   "result": {
///       "isSuccess": false,
///        "code": "1099",
///        "msg": "产线【001】不存在"
///   },
///   "targetUrl": null,
///   "success": true,
///   "error": null,
///   "unAuthorizedRequest": false,
///   "__abp": true
/// }
/// //校验成功
/// {
///   "result": {
///       "isSuccess": true,
///       "code": "200",
///       "msg": ""
///   },
///   "targetUrl": null,
///   "success": true,
///   "error": null,
///   "unAuthorizedRequest": false,
///   "__abp": true
/// }
pub async fn save_produce_test_by_auto_test<'a>(cfg: &Cfg, token: &str, req: &SaveProduceTestByAutoTestReq<'a>) -> e_utils::AnyResult<String> {
  let url = format!("{}/api/services/test/TestScan/SaveProduceTestByAutoTest", cfg.prodUrl);
  let headers = header::HeaderMap::from_iter([header(&cfg.tokenName, token)?, header("Content-Type", "application/json")?]);
  let response = reqwest::Client::new()
    .post(url)
    .headers(headers)
    .json(&json!( {
      "lotSnCode": req.lotSnCode,
      "isPass": req.isPass,
      "workCenter": &cfg.workCenter,
      "stationCode": &cfg.stationCode,
      "testFrameIdNo": &cfg.testFrameidNo,
      "testTime": req.testTime,
      "workerNo": req.workerNo,
      "keyInfo": req.keyInfo,
      "autoTestVersion": req.autoTestVersion,
      "testStartDate": req.testStartDate,
      "testEndDate": req.testEndDate,
      "testItem": req.testItem,
    }))
    .send()
    .await?
    .json::<Response<Option<ErrorResultData>>>()
    .await?;

  match (response.success, response.result, response.error) {
    (true, Some(res), _) if res.isSuccess => Ok(res.msg),
    (true, Some(res), _) => Err(res.to_str()?.into()),
    (_, _, Some(err)) => Err(err.to_str()?.into()),
    _ => Err("未知错误".into()),
  }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GetInfoBySnRes {
  /// 唯一标识
  pub id: String,
  /// 订单ID
  pub orderId: String,
  /// 订单编号
  pub orderNo: String,
  /// 批次号
  pub moLotNo: String,
  /// 订单数量
  pub orderQty: f32,
  /// 数量
  pub qty: i32,
  /// 组件基础ID
  pub componentBasisId: String,
  /// 组件基础代码
  pub componentBasisCode: String,
  /// 序列号
  pub sn: String,
  /// SN码
  pub snCode: String,
  /// 卷轴ID
  pub reelId: String,
  /// SN序号
  pub snNum: i32,
  /// SN数量
  pub snQty: f32,
  /// 内部编号
  pub innerNum: f32,
  /// 子类型
  pub subType: String,
  /// 零件ID
  pub partId: i64,
  /// 零件编号
  pub partNo: String,
  /// 产品名称
  pub productAbName: String,
  /// 产品型号
  pub productModel: String,
  /// 是否拼图
  pub isPuzzle: bool,
  /// 序列号规则集ID
  pub serialNoRuleSetId: String,
  /// 序列号头ID
  pub serialNoHeadId: String,
  /// 销售型号
  pub sellModel: String,
  /// 电气参数
  pub electricParam: String,
  /// 零件名称
  pub partName: String,
}

/// # 根据条码获取批次
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
/// #校验异常
/// ```
/// {
///   "result": {
///       "isSuccess": false,
///       "code": "400",
///       "msg": "异常信息"
///   },
///   "targetUrl": null,
///   "success": true,
///   "error": null,
///   "unAuthorizedRequest": false,
///   "__abp": true
/// }
/// //校验成功
/// {
///   "result": {
///      "isSuccess": true,
///      "code": "200",
///      "msg": "处理成功",
///      "data": {
///        "id": "1fccdf4ddb88485da4e3621a95427c9b",
///        "orderId": "9e25d30ff253461dafe970ce3d3d7669",
///        "orderNo": "QXA-SKZBU23070429",
///        "moLotNo": "QXA-SKZBU23070429",
///        "orderQty": 4212,
///        "qty": 4212,
///        "componentBasisId": "2d8c0b8f802444bf98999e52645254bf",
///        "componentBasisCode": "CMP_GW",
///        "sn": "U23070429-0A03693",
///        "snCode": "U23070429-0A03693",
///        "reelId": "U23070429-0A03693",
///        "snNum": 3693,
///        "snQty": 1,
///        "innerNum": 0,
///        "subType": "MES_COMPONENT_TYPE_ZJ",
///        "partId": 1297155,
///        "partNo": "002.002.0223135",
///        "productAbName": "TPD.SK318CL.PB751C",
///        "productModel": "TPD.SK318CL.PB751",
///        "isPuzzle": false,
///        "serialNoRuleSetId": "fc07e05f80de4e14aebce2f1ae8468be",
///        "serialNoHeadId": "8ab7318d3bcb45c7b188ab5ed940087c",
///        "sellModel": "TPD.SK318CL.PB751（T）",
///        "electricParam": "",
///        "partName": "TPD.SK318CL.PB751（T）C(22488),<10.01.0005768.002>"
///      }
///    },
///   "targetUrl": null,
///   "success": true,
///   "error": null,
///   "unAuthorizedRequest": false,
///   "__abp": true
/// }
pub async fn get_info_by_sn(cfg: &Cfg, token: &str, req: &str) -> e_utils::AnyResult<GetInfoBySnRes> {
  let url = format!("{}/api/services/test/TestScan/GetInfoBySn", cfg.prodUrl);
  let headers = header::HeaderMap::from_iter([header(&cfg.tokenName, token)?]);
  let response = reqwest::Client::new()
    .get(url)
    .headers(headers)
    .query(&[("code", req)])
    .send()
    .await?
    .json::<Response<Option<ErrorResultData>>>()
    .await?;
  match (response.success, response.result, response.error) {
    (true, Some(res), _) if res.isSuccess => Ok(serde_json::from_value(res.data)?),
    (true, Some(res), _) => Err(res.to_str()?.into()),
    (_, _, Some(err)) => Err(err.to_str()?.into()),
    _ => Err("未知错误".into()),
  }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GetProduceDetailsReq<'a> {
  /// 条码/SN code
  pub produceTestId: &'a str,
  /// 是否通过/Pass or not
  pub maxResultCount: &'a str,
}

/// # 查询测试项结果
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
/// #校验异常
pub async fn get_produce_details<'a>(cfg: &Cfg, token: &str, req: &GetProduceDetailsReq<'a>) -> e_utils::AnyResult<String> {
  let url = format!("{}/api/services/test/TestScan/GetProduceDetails", cfg.prodUrl);
  let headers = header::HeaderMap::from_iter([header(&cfg.tokenName, token)?]);
  let response = reqwest::Client::new()
    .post(url)
    .headers(headers)
    .query(&[("produceTestId", req.produceTestId), ("maxResultCount", req.maxResultCount)])
    .send()
    .await?
    .json::<Response<Option<ErrorResultData>>>()
    .await?;

  match (response.success, response.result, response.error) {
    (true, Some(res), _) if res.isSuccess => Ok(res.data.to_string()),
    (true, Some(res), _) => Err(res.to_str()?.into()),
    (_, _, Some(err)) => Err(err.to_str()?.into()),
    _ => Err("未知错误".into()),
  }
}
