use e_utils::fs::AutoPath as _;
use e_utils::AnyResult;
use std::{fs::File, io::Write as _, path::Path};

/// 写入bytes
pub fn write_bytes_file(path: impl AsRef<Path>, data: &[u8]) -> AnyResult<()> {
  let path = path.as_ref();
  if !path.exists() {
    path.auto_create_dir()?;
    // 将写入临时文件
    let mut temp_file = File::create(path)?;
    temp_file.write_all(data)?;
  }
  Ok(())
}
