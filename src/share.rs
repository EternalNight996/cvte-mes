use std::{fs::File, io::Write as _, path::Path};

use crate::input::Opts;
use e_utils::{fs::AutoPath as _, system::cmd::CmdResult};

/// 系统ROOT
pub const SYSTEM_WIN32: &'static str = "C:\\windows\\system32";
///
pub type CmdRes = CmdResult<Opts>;
use e_utils::Result;

/// 写入bytes
pub fn write_bytes_file(path: impl AsRef<Path>, data: &[u8]) -> Result<()> {
  let path = path.as_ref();
  if !path.exists() {
    path.auto_create_dir()?;
    // 将写入临时文件
    let mut temp_file = File::create(path)?;
    temp_file.write_all(data)?;
  }
  Ok(())
}

/// 初始化
pub fn default_cmd_res() -> CmdRes {
  CmdRes {
    content: String::new(),
    status: false,
    opts: Opts::default(),
  }
}

/// 初始化
pub fn cmd_res(opts: Opts) -> CmdRes {
  CmdRes {
    content: String::new(),
    status: false,
    opts,
  }
}
