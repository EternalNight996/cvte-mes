#![allow(non_snake_case)]
use e_utils::AnyResult;

use super::{Cfg, Response};

/// # 获取Token
/// ```
/// 密码加密处理
/// 账号不带@的用Base64加密，否则用MD5加密
/// 测试账号
/// 账号:chenqiang@qx.com,密码:123456
/// loginUrl地址
/// 测试：http://cmes-test.sys.gz.cvte.cn
/// uat:http://cmesqxauatsys.cvte.com
/// 正式:http://工厂短码+"-cmessys.gz.cvte.cn"
/// ```
///
/// # 请求成功，登录失败，success=false，error返回错误信息
/// ```
/// {
///     "result": null,
///     "targetUrl": null,
///     "success": false,
///     "error": {
///         "code": 0,
///         "message": "用户名或密码错误",
///         "details": null,
///         "validationErrors": null
///     },
///     "unAuthorizedRequest": false,
///     "__abp": true
/// }
/// // 请求成功，登录成功，success=false，result为token,后面所有参数带上
/// {
///     "result": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJVc2VySWQiOjQsIlRlbmFudElkIjo5LCJJbXBlcnNvbmF0b3JUZW5hbnRJZCI6bnVsbCwiSW1wZXJzb25hdG9yVXNlcklkIjpudWxsLCJVc2VyTmFtZSI6IueuoeeQhuWRmC1RWCIsIkFjY291bnQiOiJBZG1pbiIsIkV4cGlyZXNUaW1lIjoiMjAyMi0xMS0wN1QxMToxOTo0OS4wOTY4MTA2KzA4OjAwIiwiSWQiOiIwYThjNGI1ZWZlZmM0Yjc0ODQyM2JlYjBkZmJiOTVjZiIsIkxvZ2luVHlwZSI6MSwiRW1haWwiOiI5OTM5ODIwNzdAcXEuY29tIiwiVGVuYW5jeU5hbWUiOiJRWCJ9.XU9aW8JyE7hgf6tiTOL6ncxvvHK9_oFFClAKw_Gus5k",
///     "targetUrl": null,
///     "success": true,
///     "error": null,
///     "unAuthorizedRequest": false,
///     "__abp": true
/// }
/// ```
pub async fn login(cfg: &Cfg) -> AnyResult<String> {
  let url = format!("{}/api/Account/CreateToken", cfg.sysUrl);
  // 账号不带@的用Base64加密，否则用MD5加密
  let passwd = if cfg.userName.contains("@") {
    format!("{:x}", md5::compute(cfg.passWord.as_bytes()))
  } else {
    e_utils::algorithm::base64::encode(&cfg.passWord)
  };
  let response = reqwest::Client::new()
    .get(url)
    .query(&[("Email", &cfg.userName), ("Pwd", &passwd)])
    .send()
    .await
    .map_err(|_| format!("CMES(login): 无法链接 {} 请检查网络状态!", cfg.sysUrl))?
    .json::<Response<Option<String>>>()
    .await?;
  match (response.success, response.result, response.error) {
    (true, Some(token), _) => Ok(token),
    (_, _, Some(e)) => Err(e.message.into()),
    _ => Err("登录失败，未知错误".into()),
  }
}
