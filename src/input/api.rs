use super::{Opts, OptsApi};
use crate::mes::*;
use e_utils::{Error, Result};

/// # Input api 统一接口
///
/// # Rust
///```rust
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
pub async fn api(op: Opts) -> Result<String> {
  match op.api {
    OptsApi::SCM => Err(Error::String("待开发 SCM".to_string())),
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
          cmes::test_scan::get_info_by_sn(&cfg, &token, req).await
        }
        "TestScan:SaveProduceTestByAutoTest" => {
          let req: cmes::test_scan::SaveProduceTestByAutoTestReq<'_> = op
            .args
            .get(0)
            .and_then(|v| serde_json::from_str(v).ok())
            .ok_or("请把条码输入在--args 条码")?;
          let token = cmes::login::login(&cfg).await?;
          cmes::test_scan::save_produce_test_by_auto_test(&cfg, &token, &req).await
        }
        "TestScan:GetProduceDetails" => {
          let req: cmes::test_scan::GetProduceDetailsReq<'_> = op
            .args
            .get(0)
            .and_then(|v| serde_json::from_str(v).ok())
            .ok_or("请把条码输入在--args 条码")?;
          let token = cmes::login::login(&cfg).await?;
          cmes::test_scan::get_produce_details(&cfg, &token, &req).await
        }
        "Record:InsertAtSpi" => {
          let req: cmes::record::InsertAtSpiReq<'_> = op
            .args
            .get(0)
            .and_then(|v| serde_json::from_str(v).ok())
            .ok_or("请把条码输入在--args 条码")?;
          let token = cmes::login::login(&cfg).await?;
          cmes::record::insert_at_spi(&cfg, &token, &req).await
        }
        "CreateToken" => cmes::login::login(&cfg).await,
        _ => Err(Error::String("Unknow Task ".to_string())),
      }
    }
    OptsApi::None => Err(Error::String("Api cannot eq None".to_string())),
  }
}
