use cmes::test_scan::{get_info_by_sn, GetInfoBySnRes};

use super::{Opts, OptsApi};
use crate::mes::*;

/// # Input api 统一接口
///
/// # Rust
///```ignore
/// fn main() -> e_utils::Result<()> {
///   if cvte_mes::input::Opts::check_empty() {
///     let _ = app::MyApp::launch()?;
///   } else {
///     let opts = cvte_mes::input::Opts::new(None as Option<Vec<String>>)?;
///     let res = cvte_mes::input::api(opts)?;
///     println!("{}", res.to_str()?);
///   }
///   Ok(())
/// }
/// ```
pub async fn api(op: Opts) -> e_utils::AnyResult<String> {
  match op.api {
    OptsApi::SCM => {
      let cfg: crate::mes::scm::wip::Cfg = serde_json::from_str(&op.setting)?;
      match &*op.task {
        "SCM:MoFactory" => {
          let req = op
            .args
            .get(0)
            .map(|v| serde_json::from_str::<crate::mes::scm::wip::MoFactoryReq>(v))
            .transpose()?
            .ok_or("请把入参 json 放在 --args")?;
          let data = crate::mes::scm::wip::query_mo_factory(&cfg, &req).await?;
          Ok(serde_json::to_string(&data)?)
        }
        "SCM:ReqCommon" => {
          let req = op
            .args
            .get(0)
            .map(|v| serde_json::from_str::<crate::mes::scm::wip::FeedReq>(v))
            .transpose()?
            .ok_or("请把入参 json 放在 --args")?;
          let data = crate::mes::scm::wip::query_req_common(&cfg, &req).await?;
          Ok(serde_json::to_string(&data)?)
        }
        _ => Err("Unknow Task ".into()),
      }
    }
    OptsApi::ItAPI => {
      let cfg: crate::mes::itapi::Cfg = serde_json::from_str(&op.setting)?;
      match &*op.task {
        "ItAPI:BatchProdDocs" => {
          let reqs = op
            .args
            .get(0)
            .map(|v| serde_json::from_str::<Vec<crate::mes::itapi::ProdDocReq>>(v))
            .transpose()?
            .ok_or("请把入参 json 数组放在 --args")?;
          let data = crate::mes::itapi::batch_production_documents(&cfg, &reqs).await?;
          Ok(serde_json::to_string(&data)?)
        }
        "ItAPI:SoftInfoListV2" => {
          let req = op
            .args
            .get(0)
            .map(|v| serde_json::from_str::<crate::mes::itapi::SoftInfoReq>(v))
            .transpose()?
            .ok_or("请把入参 json 放在 --args")?;
          let data = crate::mes::itapi::soft_info_list_v2(&cfg, &req).await?;
          Ok(serde_json::to_string(&data)?)
        }
        "ItAPI:MacQuery" => {
          // setting 用 mes::mac::MacCfg（query_url/station_code/apikey）
          let cfg: crate::mes::mac::MacCfg = serde_json::from_str(&op.setting)?;
          let sn = op.args.get(0).ok_or("请把 SN 放在 --args")?;
          let data = crate::mes::mac::query_macs_by_sn(&cfg, None, sn).await?;
          Ok(serde_json::to_string(&data)?)
        }
        _ => Err("Unknow Task ".into()),
      }
    }
    OptsApi::CMES => {
      let cfg: cmes::Cfg = serde_json::from_str(&op.setting)?;
      match &*op.task {
        "TestScan:CheckLotSnCode" => {
          let lot_sn_code = op.args.get(0).ok_or("请把条码输入在--args 条码")?;
          let token = cmes::login::login(&cfg).await?;
          cmes::test_scan::check_lot_sn_code(&cfg, &token, lot_sn_code).await
        }
        "TestScan:GetInfoBySn" => {
          let req: &String = op.args.get(0).ok_or("请把条码输入在--args 条码")?;
          let token = cmes::login::login(&cfg).await?;
          cmes::test_scan::get_info_by_sn(&cfg, &token, req).await.and_then(|v| v.to_str())
        }
        "TestScan:SaveProduceTestByAutoTest" => {
          let arg = &op.args.join(" ");
          let req: cmes::test_scan::SaveProduceTestByAutoTestReq<'_> = serde_json::from_str(arg)?;
          let token = cmes::login::login(&cfg).await?;
          cmes::test_scan::save_produce_test_by_auto_test(&cfg, &token, &req).await
        }
        "TestScan:GetProduceDetails" => {
          let req: cmes::test_scan::GetProduceDetailsReq<'_> =
            op.args.get(0).and_then(|v| serde_json::from_str(v).ok()).ok_or("请把条码输入在--args 条码")?;
          let token = cmes::login::login(&cfg).await?;
          cmes::test_scan::get_produce_details(&cfg, &token, &req).await
        }
        "Record:InsertAtSpi" => {
          let req = op.args.get(0).ok_or("请把条码输入在--args 条码")?;
          let token = cmes::login::login(&cfg).await?;
          let v: GetInfoBySnRes = get_info_by_sn(&cfg, &token, req).await?;
          let insert_req = cmes::record::InsertAtSpiReq {
            moLotNo: &v.moLotNo,
            snCode: req,
            ..Default::default()
          };
          cmes::record::insert_at_spi(&cfg, &token, insert_req).await
        }
        "Pack:GetOtherListBySn" => {
          let front_url = op.args.first().ok_or("请把前端 API 根放在 --args[0]（如 http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod）")?;
          let sn = op.args.get(1).ok_or("请把产品条码放在 --args[1]")?;
          let token = cmes::login::login(&cfg).await?;
          let rows = cmes::pack::get_other_list_by_sn(&cfg, &token, front_url, sn).await?;
          Ok(serde_json::to_string_pretty(&rows)?)
        }
        "Pack:GetSnPackFinishQty" => {
          let front_url = op.args.first().ok_or("请把前端 API 根放在 --args[0]（如 http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod）")?;
          let lot = op.args.get(1).ok_or("请把批次号放在 --args[1]")?;
          let token = cmes::login::login(&cfg).await?;
          let rows = cmes::pack::get_sn_pack_finish_qty(&cfg, &token, front_url, lot).await?;
          Ok(serde_json::to_string_pretty(&rows)?)
        }
        "FeedSheet:GetFeedSheetList" => {
          let front_url = op
            .args
            .first()
            .ok_or("请把前端 API 根放在 --args[0]（如 http://heg-pc-cmes.gz.cvte.cn/frontApi/prod）")?;
          let parent_id = op.args.get(1).ok_or("请把 parentId 放在 --args[1]")?;
          let mo_lot = op.args.get(2).ok_or("请把批次号或 SN 放在 --args[2]")?;
          let token = cmes::login::login(&cfg).await?;
          let rows = cmes::feed_sheet::feed_sheet_list(&cfg, &token, front_url, parent_id, mo_lot).await?;
          Ok(serde_json::to_string_pretty(&rows)?)
        }
        // ---------- 《CMES 对外接口对接标准 V1.1》第 10~12 章新增（前端 bas 接口）----------
        "FeedSheetOrder:GetFeedSheetByContainerName" => {
          // --args[0]=前端 API 根，--args[1]=扫描条码（产品条码/容器条码）
          let front_url = op.args.first().ok_or("请把前端 API 根放在 --args[0]（如 http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod）")?;
          let container = op.args.get(1).ok_or("请把扫描条码放在 --args[1]")?;
          let token = cmes::login::login(&cfg).await?;
          let rows = cmes::bas::feed_sheet_by_container_name(&cfg, &token, front_url, container).await?;
          Ok(serde_json::to_string_pretty(&rows)?)
        }
        "ScanHistory:GetStationPassQty" => {
          // --args[0]=前端 API 根，--args[1]=批次号，--args[2]=工步编码（可选，逗号分隔，最多 50 个）
          let front_url = op.args.first().ok_or("请把前端 API 根放在 --args[0]（如 http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod）")?;
          let lot = op.args.get(1).ok_or("请把批次号放在 --args[1]")?;
          let codes = op.args.get(2).map(String::as_str);
          let token = cmes::login::login(&cfg).await?;
          let rows = cmes::bas::station_pass_qty(&cfg, &token, front_url, lot, codes).await?;
          Ok(serde_json::to_string_pretty(&rows)?)
        }
        "ScanHistory:VerifyStationPassQty" => {
          // 闭环验收：--args[0]=前端 API 根，--args[1]=批次号；
          // --args[2]=工站编码（可选，逗号分隔，最多 50 个）
          //   不传 -> 基线为该批次**全部工站**，并自动抽 1 个工站做闭环复查
          //   传   -> 用指定工站做闭环复查（先查全量，再查指定，逐项比对）
          let front_url = op.args.first().ok_or("请把前端 API 根放在 --args[0]（如 http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod）")?;
          let lot = op.args.get(1).ok_or("请把批次号放在 --args[1]")?;
          let codes = op.args.get(2).map(String::as_str);
          let token = cmes::login::login(&cfg).await?;
          let report = cmes::bas::verify_station_pass_qty(&cfg, &token, front_url, lot, codes).await?;
          let text = serde_json::to_string_pretty(&report)?;
          // 验收结论非 PASS 时必须让进程失败，否则渲染器只看 status 会把 FAIL 显示成 PASS
          if report.verdict == "PASS" {
            Ok(text)
          } else {
            let failed: Vec<String> = report
              .checks
              .iter()
              .filter(|c| !c.pass)
              .map(|c| format!("{}：{}", c.name, c.detail))
              .collect();
            Err(format!(
              "过站数闭环验收未通过（verdict={}）；未通过项：{}；完整报告：{text}",
              report.verdict,
              if failed.is_empty() { "（无）".to_string() } else { failed.join(" | ") }
            )
            .into())
          }
        }
        "ScanHistory:GetStationPassQtyRaw" => {
          // 【官方原样模式】完全按官方 curl 的形式发一次请求，并原样返回响应：
          //   GET {front}/api/services/bas/ScanHistory/GetStationPassQty?moLotNo=..&stationCodes=B10%2CB20
          //   Accept: application/json
          //   x-auth-token: {Token}          <- 固定用标准头名，不取 cfg.tokenName / frontTokenName
          // 不做库内的分层解析、不改写错误，便于与 curl 输出逐字节比对。
          // --args[0]=前端 API 根，--args[1]=批次号，--args[2]=工站编码（可选，逗号分隔）
          let front_url = op.args.first().ok_or("请把前端 API 根放在 --args[0]（如 http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod）")?;
          let lot = op.args.get(1).ok_or("请把批次号放在 --args[1]")?;
          let codes = op.args.get(2).map(|s| s.trim()).filter(|s| !s.is_empty());
          let token = cmes::login::login(&cfg).await?;

          let base = format!(
            "{}/api/services/bas/ScanHistory/GetStationPassQty",
            front_url.trim().trim_end_matches('/')
          );
          // 与 curl --get --data-urlencode 相同的编码规则，拼出来给现场直接对照
          let mut full_url = format!("{}?moLotNo={}", base, form_encode(lot.trim()));
          if let Some(c) = codes {
            full_url.push_str(&format!("&stationCodes={}", form_encode(c)));
          }

          let mut req = cmes::client()
            .get(&base)
            .header("Accept", "application/json")
            .header("x-auth-token", &token)
            .query(&[("moLotNo", lot.trim())]);
          if let Some(c) = codes {
            req = req.query(&[("stationCodes", c)]);
          }
          let resp = req
            .send()
            .await
            .map_err(|e| format!("官方原样模式请求失败（网络层）：{e}"))?;
          let status = resp.status().as_u16();
          let raw = resp.text().await.unwrap_or_default();
          Ok(serde_json::to_string_pretty(&serde_json::json!({
            "mode": "official（x-auth-token + 原始响应，未做分层解析）",
            "request_url": full_url,
            "auth_header": "x-auth-token",
            "accept": "application/json",
            "http_status": status,
            "raw_body": raw,
          }))?)
        }
        "SerialNoCache:GetInfoBySn" => {
          // --args[0]=前端 API 根，--args[1]=条码；--args[2]=isIncludeOrder（可选 true/false，缺省用服务端默认）；
          // --args[3]=componentBasisCode（可选，条码类型编码）
          let front_url = op.args.first().ok_or("请把前端 API 根放在 --args[0]（如 http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod）")?;
          let sn = op.args.get(1).ok_or("请把条码放在 --args[1]")?;
          let is_include_order = match op.args.get(2).map(|s| s.trim().to_lowercase()).as_deref() {
            None | Some("") => None,
            Some("true") => Some(true),
            Some("false") => Some(false),
            Some(other) => return Err(format!("--args[2] isIncludeOrder 需为 true/false，实际为 {other}").into()),
          };
          let code = op.args.get(3).map(String::as_str);
          let token = cmes::login::login(&cfg).await?;
          let info = cmes::bas::serial_no_info(&cfg, &token, front_url, sn, is_include_order, code).await?;
          Ok(serde_json::to_string_pretty(&info)?)
        }
        "CMES:MoLotNo" => {
          let sn = op.args.first().ok_or("请把条码放在 --args[0]")?;
          cmes::mo_lot_no(&cfg, sn).await
        }
        "CMES:SessionToken" => {
          let session = cmes::session::Session::new(cfg.clone());
          session.token().await
        }
        "CMES:Report" => {
          let arg = &op.args.join(" ");
          let req: cmes::test_scan::SaveProduceTestByAutoTestReq<'_> = serde_json::from_str(arg)?;
          cmes::report(&cfg, &req).await
        }
        "SelfTest:Session" => {
          let (all_pass, detail) = crate::selftest::run_all().await;
          let text = serde_json::to_string_pretty(&detail)?;
          if all_pass {
            Ok(text)
          } else {
            Err(format!("自检未通过：{text}").into())
          }
        }
        "Diagnose:Auth" => {
          // --args[0]=要探测的 URL；--args[1..]=额外候选鉴权头名（试现场真实头名，如 cmes_heg_info）
          let url = op.args.first().ok_or("请把要探测的 URL 放在 --args[0]")?;
          let extra: Vec<String> = op
            .args
            .iter()
            .skip(1)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
          let token = cmes::login::login(&cfg).await?;
          let mut attempts: Vec<serde_json::Value> = Vec::new();
          // (展示用标签, 实际请求头名, 请求头值)
          let mut candidates: Vec<(String, String, String)> = vec![
            ("x-auth-token".to_string(), "x-auth-token".to_string(), token.clone()),
            ("Authorization".to_string(), "Authorization".to_string(), format!("Bearer {token}")),
            (cfg.tokenName.clone(), cfg.tokenName.clone(), token.clone()),
            (
              format!("Cookie:{}", cfg.tokenName),
              "Cookie".to_string(),
              format!("{}={}", cfg.tokenName, token),
            ),
            ("token".to_string(), "token".to_string(), token.clone()),
          ];
          // 追加调用方指定的候选头名：既试「同名请求头」，也试「同名 Cookie」（现场两种都有先例）
          for name in extra {
            if candidates.iter().any(|(_, h, _)| *h == name) {
              continue;
            }
            candidates.push((name.clone(), name.clone(), token.clone()));
            candidates.push((format!("Cookie:{name}"), "Cookie".to_string(), format!("{name}={token}")));
          }
          let mut ok_candidates: Vec<String> = Vec::new();
          for (label, header_name, value) in candidates {
            let (code, snippet, resp_headers) = match cmes::client().get(url).header(header_name.as_str(), value).send().await {
              Ok(resp) => {
                let code = resp.status().as_u16();
                // 先取响应头再消费 body：401 时 WWW-Authenticate / Set-Cookie(名) / Location
                // 往往直接暴露前端认哪种凭据；Set-Cookie 只留 cookie 名，会话值不落日志
                let mut kept: Vec<String> = Vec::new();
                for (name, value) in resp.headers() {
                  let n = name.as_str().to_ascii_lowercase();
                  let v = value.to_str().unwrap_or("").trim();
                  if n == "set-cookie" {
                    let cookie_name = v.split('=').next().unwrap_or("").trim();
                    if !cookie_name.is_empty() {
                      kept.push(format!("set-cookie:{cookie_name}"));
                    }
                  } else if n == "www-authenticate" || n == "location" || n.starts_with("x-") {
                    let cut: String = v.chars().take(80).collect();
                    if !cut.is_empty() {
                      kept.push(format!("{n}:{cut}"));
                    }
                  }
                }
                let text = resp.text().await.unwrap_or_default();
                let cut: String = text.trim().chars().take(200).collect();
                (code, cut, kept)
              }
              Err(e) => (0u16, format!("请求失败: {e}"), Vec::new()),
            };
            let mut attempt = serde_json::json!({ "header": label.clone(), "http": code, "body": snippet });
            if !resp_headers.is_empty() {
              attempt["resp_headers"] = serde_json::json!(resp_headers);
            }
            attempts.push(attempt);
            // 汇总返回 200 的候选，便于一眼看出前端认哪种凭据（无需逐条扫 attempts）
            if code == 200 {
              ok_candidates.push(label);
            }
          }
          Ok(serde_json::to_string_pretty(&serde_json::json!({
            "url": url,
            "note": "依次用不同鉴权头访问同一 URL；看哪种返回 200，即可判断前端 API 认什么鉴权（Token 值不落日志）。resp_headers 只保留 WWW-Authenticate / Location / x-* 以及与 Set-Cookie 的 cookie 名（不保留会话值）",
            "login_token_ok": true,
            "ok_candidates": ok_candidates,
            "attempts": attempts
          }))?)
        }
        "User:CreateToken" => cmes::login::login(&cfg).await,
        _ => Err("Unknow Task ".into()),
      }
    }
    OptsApi::None => Err("Api cannot eq None".into()),
  }
}

/// 按 `application/x-www-form-urlencoded` 规则编码，与 `curl --get --data-urlencode` 一致：
/// 字母/数字与 `-_.~` 原样保留，空格变成 `+`，其余（含逗号）编码成 `%XX`。
///
/// 用途：`ScanHistory:GetStationPassQtyRaw` 会把「官方拼法」的完整 URL 打印出来，
/// 现场可直接与 curl 命令对照，确认两边发出的字节相同。
fn form_encode(s: &str) -> String {
  let mut out = String::with_capacity(s.len() * 3);
  for b in s.bytes() {
    match b {
      b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
      b' ' => out.push('+'),
      _ => out.push_str(&format!("%{b:02X}")),
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::form_encode;

  #[test]
  fn form_encode_matches_curl_data_urlencode() {
    // curl --get --data-urlencode "stationCodes=B10,B20" 会编码成 B10%2CB20
    assert_eq!(form_encode("B10,B20"), "B10%2CB20");
    assert_eq!(form_encode("B10"), "B10");
    assert_eq!(form_encode("HEG-YKCBU26090494"), "HEG-YKCBU26090494");
    // 空格按 form 规则变 +
    assert_eq!(form_encode("A B"), "A+B");
  }
}