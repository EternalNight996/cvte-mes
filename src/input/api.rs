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
        "User:CreateToken" => cmes::login::login(&cfg).await,
        _ => Err("Unknow Task ".into()),
      }
    }
    OptsApi::None => Err("Api cannot eq None".into()),
  }
}
