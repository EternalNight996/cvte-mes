/// API
pub mod api;
pub use api::*;
use e_utils::{Error, Result};
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use structopt::{clap::arg_enum, StructOpt};

arg_enum! {
    /// Api接口
    #[derive(Deserialize, Serialize, Debug, StructOpt, Clone, PartialEq, Copy)]
    pub enum OptsApi {
        None,
        SCM,
        CMES
    }
}

/// cvte-mes
///
/// ------------------------------------------------------
///
///
#[derive(StructOpt, Debug, Clone, Serialize, Deserialize)]
#[structopt(name = "", setting = structopt::clap::AppSettings::TrailingVarArg,)]
#[structopt(after_help = r#"
# Example
-----------------------------------------------------------
# Cmd Example Active SYSTEM
```sh
cvte-mes.exe --api active --task rkms -- kms.03k.org
cvte-mes.exe --api active --task active -- YVWGF-BXNMC-HTQYQ-CPQ99-66QFC
cvte-mes.exe --api active --task check
```
-----------------------------------------------------------

# Cmd Example deActive SYSTEM
```sh
cvte-mes.exe --api active --task ckms
cvte-mes.exe --api active --task deactive
cvte-mes.exe --api active --task check
```
-----------------------------------------------------------

# Cmd Example empty task
```sh
cvte-mes.exe --api clean --task empty_recycle_bin
cvte-mes.exe --api clean --task empty_access_log
cvte-mes.exe --api clean --task empty_netshare
cvte-mes.exe --api clean --task empty_activity_history
cvte-mes.exe --api clean --task empty_run_history
```
-----------------------------------------------------------

# Cmd Example Drive commands
```sh
# Find drive from node
cvte-mes.exe --init --api Drive --full --task findnodes --filter "Intel(R) Ethernet Controller (3) I225-V #4" -- =net
# Remove drive
cvte-mes.exe --init --api Drive --full --task remove --args /force --filter "Intel(R) Ethernet Controller (3) I225-V #4" -- =net
# Disable drive
cvte-mes.exe --init --api Drive --full --task disable --args /force --filter "Intel(R) Ethernet Controller (3) I225-V #4" -- =net
# Enable drive
cvte-mes.exe --init --api Drive --full --task enable --args /force --filter "Intel(R) Ethernet Controller (3) I225-V #4" -- =net
# Restart drive
cvte-mes.exe --init --api Drive --full --task restart --args /force --filter "Intel(R) Ethernet Controller (3) I225-V #4" -- =net
# Scan all new drive
cvte-mes.exe --api Drive --task scan
# Delete drive throgh node info : /reboot
cvte-mes.exe --init --api Drive --full --task delete-find --args /force --filter "Intel(R) Ethernet Controller (3) I225-V #4" -- =net
# Delete with uninstall drive throgh inf file : /reboot
cvte-mes.exe --api Drive --task delete --args F:\device.inf /uninstall /force
# Add drive throgh inf file : /reboot
cvte-mes.exe --api Drive --task add --args F:\device.inf /install /force
# Add more drive throgh inf file : /reboot
cvte-mes.exe --api Drive --task add-file --args F:\drives /install /force
# Output more or one drive <oem#.inf | *> <target directory>
cvte-mes.exe --api Drive --task export --args * F:\drives
```
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
  pub fn new<I>(args: Option<I>) -> Result<Self>
  where
    Self: Sized,
    I: IntoIterator,
    I::Item: Into<OsString> + Clone,
  {
    match args {
      Some(arg) => match Opts::from_iter_safe(arg) {
        Ok(opt) => Ok(opt),
        Err(e) => Err(Error::String(e.to_string())),
      },
      None => Ok(Opts::from_args()),
    }
  }
  /// # 检查空
  pub fn check_empty() -> bool {
    std::env::args().len() == 1
  }
}
