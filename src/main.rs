use cvte_mes::input::api;
use cvte_mes::input::Opts;
use e_utils::{cmd::CmdResult, Result};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
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
