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
# CMES Example
```sh
# 测试登录
cargo run -- --api CMES --task User:CreateToken --setting '{\"stationName\":\"测试\",\"prodUrl\":\"http://url.cn\",\"sysUrl\":\"http://url.cn\",\"userName\":\"uname\",\"passWord\":\"passwd\",\"tokenName\":\"cmes_test_info\",\"causeNo\":\"test\",\"workCenter\":\"TEST01\",\"testFrameidNo\":\"TEST\",\"stationCode\":\"test\"}'

# 判断测试工步
cargo run -- --api CMES --task TestScan:CheckLotSnCode --args '条码' --setting '{\"stationName\":\"测试\",\"prodUrl\":\"http://url.cn\",\"sysUrl\":\"http://url.cn\",\"userName\":\"uname\",\"passWord\":\"passwd\",\"tokenName\":\"cmes_test_info\",\"causeNo\":\"test\",\"workCenter\":\"TEST01\",\"testFrameidNo\":\"TEST\",\"stationCode\":\"test\"'

# 根据条码获取批次
cargo run -- --api CMES --task TestScan:GetInfoBySn --args '条码' --setting '{\"stationName\":\"测试\",\"prodUrl\":\"http://url.cn\",\"sysUrl\":\"http://url.cn\",\"userName\":\"uname\",\"passWord\":\"passwd\",\"tokenName\":\"cmes_test_info\",\"causeNo\":\"test\",\"workCenter\":\"TEST01\",\"testFrameidNo\":\"TEST\",\"stationCode\":\"test\"}'

# 保存测试结果
cargo run -- --api CMES --task TestScan:SaveProduceTestByAutoTest --args '"{\"lotSnCode\":\"XHA241590001\",\"isPass\":true,\"testTime\":\"2024-10-28 02:19:44\",\"workerNo\":\"梦游\",\"keyInfo\":\"\",\"autoTestVersion\":\"heg-autotest2-v0.1.0\",\"testStartDate\":\"2024-10-28 02:19:44\",\"testEndDate\":\"2024-10-28 02:19:44\",\"testItem\":\"网络MAC\"}"'  --setting '{\"stationName\":\"测试\",\"prodUrl\":\"http://url.cn\",\"sysUrl\":\"http://url.cn\",\"userName\":\"uname\",\"passWord\":\"passwd\",\"tokenName\":\"cmes_test_info\",\"causeNo\":\"test\",\"workCenter\":\"TEST01\",\"testFrameidNo\":\"TEST\",\"stationCode\":\"test\"}'
# 查询测试项结果
cargo run -- --api CMES --task TestScan:GetProduceDetails --args '{\"produceTestId\":\"\",\"maxResultCount\":\"\"}' --setting '{\"stationName\":\"测试\",\"prodUrl\":\"http://url.cn\",\"sysUrl\":\"http://url.cn\",\"userName\":\"uname\",\"passWord\":\"passwd\",\"tokenName\":\"cmes_test_info\",\"causeNo\":\"test\",\"workCenter\":\"TEST01\",\"testFrameidNo\":\"TEST\",\"stationCode\":\"test\"}'

# 序列扫描过站
cargo run -- --api CMES --task Record:InsertAtSpi --args '条码' --setting '{\"stationName\":\"测试\",\"prodUrl\":\"http://url.cn\",\"sysUrl\":\"http://url.cn\",\"userName\":\"uname\",\"passWord\":\"passwd\",\"tokenName\":\"cmes_test_info\",\"causeNo\":\"test\",\"workCenter\":\"TEST01\",\"testFrameidNo\":\"TEST\",\"stationCode\":\"test\"}'```

# SCM Example
-----------------------------------------------------------
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
