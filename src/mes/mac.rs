#![allow(non_snake_case)]
//! SN→MAC 绑定查询（条码防混料）通用接口
//!
//! 支持两种鉴权：
//! - CMES 模式：`token` 请求头（token_name）
//! - ItAPI 模式：`apikey` 请求头
//!
//! 响应解析兼容两种格式：
//! - CMES 信封：`{success, result:{isSuccess, code, msg, data:[...]}, error}`
//! - 直接数组：`[{name, value}, ...]`
//!
//! 返回数量即该 SN 在 MES 上绑定的 MAC 数量，由调用方按此逐个扫描比对。
use e_utils::AnyResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::mes::cmes::{ErrorResultData, Response};

/// MES 返回的 MAC 绑定项：名称标识（WIFI MAC / LAN MAC / BLUETOOTH MAC ...）+ 值
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct MacBinding {
  pub name: String,
  #[serde(default, alias = "expected", alias = "mac")]
  pub value: String,
}

/// MAC 查询配置（对应插件配置卡：mac_query_url / 工位号 / 鉴权）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MacCfg {
  /// SN→MAC 绑定查询接口完整地址（必填）
  pub query_url: String,
  /// 工位号（请求入参 stationCode）
  pub station_code: String,
  /// CMES token 请求头名（CMES 鉴权时用）
  pub token_name: String,
  /// ItAPI 应用 ID（ItAPI apikey 鉴权时用）
  pub apikey: String,
}

/// MAC 归一化：去掉分隔符、忽略大小写（AA:BB:CC == aabbcc）
pub fn norm_mac(s: &str) -> String {
  s.trim().to_uppercase().replace([':', '-', '.', ' '], "")
}

/// ItAPI style envelope: {status, message, data}
#[derive(Deserialize, Debug, Default)]
struct ItapiResp<T> {
  /// 状态码（信封保留字段；解析只消费 data/message，见 ItAPI 约定）
  #[serde(default)]
  #[allow(dead_code)]
  status: Option<String>,
  /// 消息（信封保留字段；解析只消费 data，见 ItAPI 约定）
  #[serde(default)]
  #[allow(dead_code)]
  message: Option<String>,
  #[serde(default)]
  data: Option<T>,
}

fn parse_macs(text: &str) -> AnyResult<Vec<MacBinding>> {
  // 优先直接数组 [{name, value}]
  if let Ok(macs) = serde_json::from_str::<Vec<MacBinding>>(text) {
    return Ok(macs);
  }
  // ItAPI envelope {status, message, data:[...]}
  if let Ok(env) = serde_json::from_str::<ItapiResp<Vec<MacBinding>>>(text) {
    if let Some(data) = env.data {
      return Ok(data);
    }
  }
  // 其次 CMES 信封 {success, result:{isSuccess, msg, data:[...]}}
  if let Ok(env) = serde_json::from_str::<Response<Option<ErrorResultData>>>(text) {
    match (env.success, env.result, env.error) {
      (true, Some(r), _) if r.isSuccess => {
        let data = if r.data.is_null() {
          Value::Array(vec![])
        } else {
          r.data
        };
        return serde_json::from_value::<Vec<MacBinding>>(data)
          .map_err(|e| format!("MAC 数据解析失败: {e}").into());
      }
      (true, Some(r), _) => return Err(format!("查询 MAC 拦截: {}", r.msg).into()),
      (_, _, Some(e)) => return Err(format!("查询 MAC 异常: {}", e.message).into()),
      _ => return Err("查询 MAC 失败: 未知错误".into()),
    }
  }
  Err("MAC 响应解析失败: 无法识别的响应格式".into())
}

/// 查询 SN 绑定的 MAC 列表
///
/// `token` 传 `Some` 走 CMES token 鉴权；传 `None` 走 ItAPI apikey 鉴权。
pub async fn query_macs_by_sn(
  cfg: &MacCfg,
  token: Option<&str>,
  sn: &str,
) -> AnyResult<Vec<MacBinding>> {
  if cfg.query_url.is_empty() {
    return Err("未配置 mac_query_url（SN→MAC 接口地址）".into());
  }
  let mut builder = reqwest::Client::new()
    .post(&cfg.query_url)
    .header("Content-Type", "application/json");
  if let Some(t) = token {
    builder = builder.header(&cfg.token_name, t);
  } else if !cfg.apikey.is_empty() {
    builder = builder.header("apikey", &cfg.apikey);
  }
  let text = builder
    .json(&serde_json::json!({ "sn": sn, "stationCode": cfg.station_code }))
    .send()
    .await
    .map_err(|_| "查询 MAC: 网络异常".to_string())?
    .text()
    .await
    .map_err(|e| format!("读取响应失败: {e}"))?;
  let macs: Vec<MacBinding> = parse_macs(&text)?
    .into_iter()
    .filter(|m| !m.name.trim().is_empty() && !m.value.trim().is_empty())
    .collect();
  Ok(macs)
}

/// 演示用模拟 MAC（按 SN 用 MD5 派生，结果可重现；联调前用于预览完整流程）
pub fn mock_macs(sn: &str) -> Vec<MacBinding> {
  let digest = md5::compute(sn.as_bytes());
  let mac = |start: usize| {
    format!(
      "{:02X}-{:02X}-{:02X}-{:02X}-{:02X}-{:02X}",
      digest[start],
      digest[start + 1],
      digest[start + 2],
      digest[start + 3],
      digest[start + 4],
      digest[start + 5]
    )
  };
  vec![
    MacBinding { name: "WIFI MAC".into(), value: mac(0) },
    MacBinding { name: "LAN MAC".into(), value: mac(2) },
    MacBinding { name: "BLUETOOTH MAC".into(), value: mac(4) },
  ]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn norm_mac_ignores_separators_and_case() {
    assert_eq!(norm_mac("AA:BB:CC:DD:EE:FF"), "AABBCCDDEEFF");
    assert_eq!(norm_mac("aa-bb-cc-dd-ee-ff"), "AABBCCDDEEFF");
    assert_eq!(norm_mac(" AA.BB.CC.DD.EE.FF "), "AABBCCDDEEFF");
  }

  #[test]
  fn mock_macs_stable_and_three_kinds() {
    let a = mock_macs("SN12345678");
    let b = mock_macs("SN12345678");
    assert_eq!(a, b);
    assert_eq!(a.len(), 3);
    assert_eq!(a[0].name, "WIFI MAC");
    assert_eq!(a[1].name, "LAN MAC");
    assert_eq!(a[2].name, "BLUETOOTH MAC");
    assert_eq!(norm_mac(&a[0].value).len(), 12);
  }

  #[test]
  fn parse_direct_array() {
    let text = r#"[{"name":"WIFI MAC","value":"AA:BB:CC:DD:EE:01"}]"#;
    let macs = parse_macs(text).unwrap();
    assert_eq!(macs.len(), 1);
    assert_eq!(macs[0].name, "WIFI MAC");
    assert_eq!(macs[0].value, "AA:BB:CC:DD:EE:01");
  }

  #[test]
  fn parse_cmes_envelope() {
    let text = r#"{"success":true,"result":{"isSuccess":true,"code":"200","msg":"","data":[{"name":"LAN MAC","expected":"11-22-33-44-55-66"}]}}"#;
    let macs = parse_macs(text).unwrap();
    assert_eq!(macs.len(), 1);
    assert_eq!(macs[0].name, "LAN MAC");
    assert_eq!(macs[0].value, "11-22-33-44-55-66");
  }

  #[test]
  fn parse_itapi_envelope() {
    let text = r#"{"status":"0","message":"success","data":[{"name":"WIFI MAC","value":"AA:BB:CC:DD:EE:01"}]}"#;
    let macs = parse_macs(text).unwrap();
    assert_eq!(macs.len(), 1);
    assert_eq!(macs[0].name, "WIFI MAC");
    assert_eq!(macs[0].value, "AA:BB:CC:DD:EE:01");
  }
}
