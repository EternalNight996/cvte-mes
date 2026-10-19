/// API
pub mod api;
pub use api::*;
use e_utils::{cmd::CmdResult, AnyResult};
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use structopt::{clap::arg_enum, StructOpt};

arg_enum! {
    /// Api接口
    #[derive(Deserialize, Serialize, Debug, StructOpt, Clone, PartialEq, Copy)]
    pub enum OptsApi {
        None,
        SCM,
        CMES,
        ItAPI
    }
}

/// cvte-mes
/// ------------------------------------------------------
#[derive(StructOpt, Debug, Clone, Serialize, Deserialize)]
#[structopt(name = "", setting = structopt::clap::AppSettings::TrailingVarArg,)]
#[structopt(after_help = r#"
================================================================
参数说明
  --api     CMES | SCM | ItAPI
  --task    任务名（见下方分组）
  --args    任务入参，**顺序敏感**，每条示例都标注了对应位置
  --setting 配置 JSON
            · .bat 里内层引号写成 \"（否则 cmd 会把引号吃掉）
            · PowerShell / bash 用单引号包住即可原样传递
  FRONT     前端 API 根 = {域名}/frontApi/prod
================================================================

----- CMES · 鉴权 / 自检 -----
# 登录取 Token
cvte-mes.exe --api CMES --task User:CreateToken --setting '<CMES setting>'

# 内置自检（本地 mock，16 项；不访问生产、无业务副作用）
cvte-mes.exe --api CMES --task SelfTest:Session --setting '<CMES setting>'

# 前端 API 鉴权诊断：试多种凭据头，汇总 ok_candidates + resp_headers
#   --args[0]=要探测的 URL，--args[1..]=额外候选头名（会同时试同名请求头与同名 Cookie）
cvte-mes.exe --api CMES --task Diagnose:Auth --args '<FRONT>/api/services/pack/Pack/GetOtherListBySn?snCode=<条码>' 'cmes_heg_info' --setting '<CMES setting>'

----- CMES · 过站 / 报工 -----
# 条码过站校验（--args[0]=条码）
cvte-mes.exe --api CMES --task TestScan:CheckLotSnCode --args '<条码>' --setting '<CMES setting>'
# 按条码查批次（--args[0]=条码）
cvte-mes.exe --api CMES --task TestScan:GetInfoBySn --args '<条码>' --setting '<CMES setting>'
# 查询测试项结果（--args[0]=JSON）
cvte-mes.exe --api CMES --task TestScan:GetProduceDetails --args '{\"produceTestId\":\"<条码>\",\"maxResultCount\":\"10\"}' --setting '<CMES setting>'
# 保存测试结果 / 报工（--args[0]=JSON）
cvte-mes.exe --api CMES --task TestScan:SaveProduceTestByAutoTest --args '{\"lotSnCode\":\"<条码>\",\"isPass\":true,\"testTime\":\"2026-10-19 10:00:00\",\"workerNo\":\"<工号>\",\"keyInfo\":\"\",\"autoTestVersion\":\"cvte-mes\",\"testStartDate\":\"2026-10-19 10:00:00\",\"testEndDate\":\"2026-10-19 10:00:00\",\"testItem\":\"网络MAC\"}' --setting '<CMES setting>'
# 序列扫描过站（--args[0]=条码）
cvte-mes.exe --api CMES --task Record:InsertAtSpi --args '<条码>' --setting '<CMES setting>'
# 组合：登录->校验->报工（--args[0]=JSON，同上）
cvte-mes.exe --api CMES --task CMES:Report --args '<同上 JSON>' --setting '<CMES setting>'

----- CMES · 投料单 / 包装 / 组件 -----
# 投料单查询（--args[0]=FRONT  --args[1]=parentId  --args[2]=批次号或SN）
cvte-mes.exe --api CMES --task FeedSheet:GetFeedSheetList --args '<FRONT>' '<parentId>' '<批次号>' --setting '<CMES setting>'

# 【V1.1-10】扫描条码取所属批次的投料单物料行（--args[0]=FRONT  --args[1]=扫描条码）
cvte-mes.exe --api CMES --task FeedSheetOrder:GetFeedSheetByContainerName --args '<FRONT>' '<条码>' --setting '<CMES setting>'

# 按产品条码取组件关联列表（--args[0]=FRONT  --args[1]=产品条码）
cvte-mes.exe --api CMES --task Pack:GetOtherListBySn --args '<FRONT>' '<条码>' --setting '<CMES setting>'

# 按批次取 SN 包装完工数（--args[0]=FRONT  --args[1]=批次号）
cvte-mes.exe --api CMES --task Pack:GetSnPackFinishQty --args '<FRONT>' '<批次号>' --setting '<CMES setting>'

----- CMES · 过站数量（V1.1 第 11 章）★ -----
# ① 全部工站：不传 stationCodes，返回该批次【所有工步】的数据
cvte-mes.exe --api CMES --task ScanHistory:GetStationPassQty --args '<FRONT>' '<批次号>' --setting '<CMES setting>'

# ② 指定工站：--args[2]=工站编码，多个用英文逗号分隔，单次最多 50 个
#    只返回这些工站的数据（支持 B10 / B10,B20 等写法，带空格会自动归一化）
cvte-mes.exe --api CMES --task ScanHistory:GetStationPassQty --args '<FRONT>' '<批次号>' 'B10' --setting '<CMES setting>'
cvte-mes.exe --api CMES --task ScanHistory:GetStationPassQty --args '<FRONT>' '<批次号>' 'B10,B20' --setting '<CMES setting>'

# ③ 闭环验收：先查全部工站，再指定工站复查，逐项比对（--args[2]=抽查几个工站，默认 3）
#    检查项：只返回指定工站 / 过站数与全量一致 / 行数一致 / 抽查须收窄 /
#            moLotNo 与请求一致 / moQty 一致 / sortNo 升序 / 同工站整机在部件前
#    verdict=PASS 才算通过；NO_DATA 表示该批次无过站数据；非 PASS 时进程退出码非 0
cvte-mes.exe --api CMES --task ScanHistory:VerifyStationPassQty --args '<FRONT>' '<批次号>' '1' --setting '<CMES setting>'

----- CMES · 条码详细信息（V1.1 第 12 章）-----
# 默认（服务端默认 isIncludeOrder=true）
cvte-mes.exe --api CMES --task SerialNoCache:GetInfoBySn --args '<FRONT>' '<条码>' --setting '<CMES setting>'
# 带可选参数：--args[2]=isIncludeOrder(true/false)  --args[3]=componentBasisCode(条码类型编码)
cvte-mes.exe --api CMES --task SerialNoCache:GetInfoBySn --args '<FRONT>' '<条码>' 'false' 'CMP_INNERBOX' --setting '<CMES setting>'

----- SCM（MD5 网关签名）-----
cvte-mes.exe --api SCM --task SCM:MoFactory --args '{\"lineLotNumber\":\"<线体批次>\"}' --setting '{\"gateway\":\"https://itgw-sit.cvte.com/env-101/scm/scmwip/gw-wip\",\"apikey\":\"<应用ID>\",\"api_secret\":\"<密钥>\"}'
cvte-mes.exe --api SCM --task SCM:ReqCommon --args '{\"lineLotNumberList\":[\"<线体批次>\"],\"lotNumberList\":[\"<批次号>\"]}' --setting '<SCM setting>'

----- ItAPI -----
cvte-mes.exe --api ItAPI --task ItAPI:BatchProdDocs --args '[{\"docSubClassList\":[],\"prodNumber\":\"<物料号>\"}]' --setting '{\"base_url\":\"https://itapi-portal.cvte.com\",\"apikey\":\"<应用ID>\"}'
cvte-mes.exe --api ItAPI --task ItAPI:SoftInfoListV2 --args '{\"lotNumbers\":[],\"moLineIds\":[]}' --setting '<ItAPI setting>'
cvte-mes.exe --api ItAPI --task ItAPI:MacQuery --args '<SN>' --setting '{\"base_url\":\"<ItAPI>\",\"apikey\":\"<应用ID>\",\"query_url\":\"<SN→MAC 地址>\",\"station_code\":\"<工位>\",\"token_name\":\"<令牌名>\"}'

================================================================
提示
  · 返回统一为 {"content": ..., "status": true/false, "opts": null}
  · status=false 表示调用失败，content 里是可直接读的原因
  · 过站数量：不传 stationCodes = 该批次全部工站；传 = 只返回指定工站
  · 条码不存在时 SerialNoCache:GetInfoBySn 返回 content:null 且 status:true（属正常）
  · 配置项 frontTokenName：前端 API 专用鉴权头名；留空则回退 tokenName
      （实测存在后端与前端要不同头名的站点：板卡后端 cmes_heg-mb_info、前端 cmes_heg_info）
  · 批量验证：scripts\verify.bat（34 个用例，含上述全部 V1.1 新增接口与闭环验收）
================================================================
"#)]
#[allow(clippy::struct_excessive_bools)]
pub struct Opts {
  /// API接口
  #[structopt(required = true, short, long, possible_values = &OptsApi::variants(), case_insensitive = true)]
  pub api: OptsApi,
  /// 任务
  #[structopt(long, required = false, default_value = "")]
  pub task: String,
  /// 配置
  #[structopt(long, required = true)]
  pub setting: String,
  /// 扩展参数
  #[structopt(long, required = false)]
  pub args: Vec<String>,
  /// 扩展指令
  #[structopt(required = false, last = true)]
  pub command: Vec<String>,
}
impl Default for Opts {
  fn default() -> Self {
    Self {
      api: OptsApi::None,
      task: String::new(),
      setting: String::new(),
      args: Vec::new(),
      command: Vec::new(),
    }
  }
}

impl Opts {
  /// # Example
  ///```ignore
  /// use cvte_mes::input::api;
  /// use cvte_mes::input::Opts;
  /// use e_utils::{cmd::CmdResult, Result};
  /// use serde_json::Value;
  /// #[tokio::main]
  /// async fn main() -> Result<()> {
  ///   let opts = Opts::new(None as Option<Vec<&str>>)?;
  ///   let mut res: CmdResult<Value> = CmdResult {
  ///     content: String::new(),
  ///     status: false,
  ///     opts: Value::Null,
  ///   };
  ///   match api(opts).await {
  ///     Ok(v) => {
  ///       res.content = v;
  ///       res.status = true;
  ///     }
  ///     Err(e) => res.content = e.to_string(),
  ///   }
  ///   println!("{}", res.to_string_pretty()?);
  ///   Ok(())
  /// }
  /// ```
  pub fn new<I>(args: Option<I>) -> AnyResult<Self>
  where
    Self: Sized,
    I: IntoIterator,
    I::Item: Into<OsString> + Clone,
  {
    match args {
      Some(arg) => match Opts::from_iter_safe(arg) {
        Ok(opt) => Ok(opt),
        Err(e) => Err(e.into()),
      },
      None => Ok(Opts::from_args()),
    }
  }
  /// # 检查空
  pub fn check_empty() -> bool {
    std::env::args().len() == 1
  }
  pub fn default_res() -> CmdRes {
    CmdRes {
      content: String::new(),
      status: false,
      opts: Opts::default(),
    }
  }
}
pub type CmdRes = CmdResult<Opts>;
