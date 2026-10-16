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
          let url = op.args.first().ok_or("请把要探测的 URL 放在 --args[0]")?;
          let token = cmes::login::login(&cfg).await?;
          let mut attempts: Vec<serde_json::Value> = Vec::new();
          let candidates: Vec<(String, String)> = vec![
            ("x-auth-token".to_string(), token.clone()),
            ("Authorization".to_string(), format!("Bearer {token}")),
            (cfg.tokenName.clone(), token.clone()),
            ("Cookie".to_string(), format!("{}={}", cfg.tokenName, token)),
            ("token".to_string(), token.clone()),
          ];
          for (name, value) in candidates {
            let (code, snippet) = match cmes::client().get(url).header(name.as_str(), value).send().await {
              Ok(resp) => {
                let code = resp.status().as_u16();
                let text = resp.text().await.unwrap_or_default();
                let cut: String = text.trim().chars().take(200).collect();
                (code, cut)
              }
              Err(e) => (0u16, format!("请求失败: {e}")),
            };
            attempts.push(serde_json::json!({ "header": name, "http": code, "body": snippet }));
          }
          Ok(serde_json::to_string_pretty(&serde_json::json!({
            "url": url,
            "note": "依次用不同鉴权头访问同一 URL；看哪种返回 200，即可判断前端 API 认什么鉴权（Token 值不落日志）",
            "login_token_ok": true,
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
