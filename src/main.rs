use cvte_mes::input::api;
use cvte_mes::input::Opts;
use e_utils::cmd::CmdResult;
use e_utils::AnyResult;
use serde_json::Value;

#[tokio::main]
async fn main() -> AnyResult<()> {
  // rustls-no-provider：进程启动即安装 crypto provider
  // （各网络模块现已统一走 cmes::client()，其内部也会幂等安装，此处为兜底）
  cvte_mes::mes::cmes::ensure_tls();
  let opts = Opts::new(None as Option<Vec<&str>>)?;
  let mut res: CmdResult<Value> = CmdResult {
    content: String::new(),
    status: false,
    opts: Value::Null,
  };
  match api(opts).await {
    Ok(v) => {
      res.content = v;
      res.status = true;
    }
    Err(e) => res.content = e.to_string(),
  }
  println!("{}", res.to_string_pretty()?);
  Ok(())
}
