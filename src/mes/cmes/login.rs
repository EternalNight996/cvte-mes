#![allow(non_snake_case)]
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
pub async fn login(cfg: &Cfg) -> e_utils::Result<String> {
  let url = format!("{}/api/Account/CreateToken", cfg.sysUrl);
  let response = reqwest::Client::new()
    .get(url)
    .query(&[("Email", &cfg.userName), ("Pwd", &cfg.passWord)])
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json::<Response<Option<String>>>()
    .await
    .map_err(|e| e.to_string())?;

  match (response.success, response.result, response.error) {
    (true, Some(token), _) => Ok(token),
    (_, _, Some(e)) => Err(e.message.into()),
    _ => Err("登录失败，未知错误".into()),
  }
}
