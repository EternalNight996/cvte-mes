//! 从 etest.db 生成 `verify.prod.bat`（现场电脑无需 Python）
//!
//! 独立小工具，由 `gen-cfg` feature 启用：`cargo build --release --features gen-cfg`
//!
//! ```text
//! cvte-mes-gencfg --list                          # 列出 etest.db 里的候选配置
//! cvte-mes-gencfg                                 # 取字段最完整的一条生成
//! cvte-mes-gencfg --id 1 --sn <SN> --lot <LOT>    # 指定记录并写入测试数据
//! cvte-mes-gencfg --db D:\etest.db --out .\verify.prod.bat
//! ```
use std::fs;
use std::path::Path;
use std::process::exit;

use rusqlite::Connection;

const DEFAULT_DB: &str = r"F:\MyApp\eternal\etest\etest.db";

struct Args {
  db: String,
  out: String,
  id: Option<i64>,
  list: bool,
  sn: Option<String>,
  sn2: Option<String>,
  lot: Option<String>,
  lot2: Option<String>,
  parent_id: Option<String>,
}

#[derive(Debug, Clone)]
struct Row {
  id: i64,
  org_tag: String,
  station_name: String,
  station_code: String,
  prod_url: String,
  sys_url: String,
  username: String,
  password: String,
  token_name: String,
  cause_no: String,
  work_center: String,
  test_frameid_no: String,
}

impl Row {
  fn usable(&self) -> bool {
    !self.prod_url.trim().is_empty() && !self.username.trim().is_empty()
  }
}

fn die(msg: &str) -> ! {
  eprintln!("[ERROR] {msg}");
  exit(1);
}

fn parse_args() -> Args {
  let exe_dir = std::env::current_exe()
    .ok()
    .and_then(|p| p.parent().map(Path::to_path_buf))
    .unwrap_or_else(|| Path::new(".").to_path_buf());
  let mut a = Args {
    db: DEFAULT_DB.to_string(),
    out: exe_dir.join("verify.prod.bat").to_string_lossy().to_string(),
    id: None,
    list: false,
    sn: None,
    sn2: None,
    lot: None,
    lot2: None,
    parent_id: None,
  };
  let argv: Vec<String> = std::env::args().skip(1).collect();
  let mut i = 0;
  while i < argv.len() {
    let take = |i: &mut usize| -> String {
      *i += 1;
      argv.get(*i).cloned().unwrap_or_default()
    };
    match argv[i].as_str() {
      "--db" => a.db = take(&mut i),
      "--out" => a.out = take(&mut i),
      "--id" => a.id = take(&mut i).parse().ok(),
      "--list" => a.list = true,
      "--sn" => a.sn = Some(take(&mut i)),
      "--sn2" => a.sn2 = Some(take(&mut i)),
      "--lot" => a.lot = Some(take(&mut i)),
      "--lot2" => a.lot2 = Some(take(&mut i)),
      "--parent-id" => a.parent_id = Some(take(&mut i)),
      "-h" | "--help" => {
        println!("用法: cvte-mes-gencfg [--db <etest.db>] [--out <verify.prod.bat>] [--id N] [--list]");
        println!("      [--sn S] [--sn2 S] [--lot L] [--lot2 L] [--parent-id P]");
        exit(0);
      }
      other => die(&format!("未知参数: {other}")),
    }
    i += 1;
  }
  a
}

fn load_rows(db: &str) -> Vec<Row> {
  if !Path::new(db).exists() {
    die(&format!("找不到 etest.db：{db}（用 --db 指定）"));
  }
  let conn = match Connection::open_with_flags(db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY) {
    Ok(c) => c,
    Err(e) => die(&format!("无法打开 {db}: {e}")),
  };
  let mut stmt = match conn.prepare(
    "select id, org_tag, station_name, station_code, prod_url, sys_url, username, password, \
     token_name, cause_no, work_center, test_frameid_no from org_cvte_cmes order by id",
  ) {
    Ok(s) => s,
    Err(e) => die(&format!("查询 org_cvte_cmes 失败: {e}")),
  };
  let rows = stmt
    .query_map([], |r| {
      Ok(Row {
        id: r.get(0)?,
        org_tag: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
        station_name: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
        station_code: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
        prod_url: r.get::<_, Option<String>>(4)?.unwrap_or_default(),
        sys_url: r.get::<_, Option<String>>(5)?.unwrap_or_default(),
        username: r.get::<_, Option<String>>(6)?.unwrap_or_default(),
        password: r.get::<_, Option<String>>(7)?.unwrap_or_default(),
        token_name: r.get::<_, Option<String>>(8)?.unwrap_or_default(),
        cause_no: r.get::<_, Option<String>>(9)?.unwrap_or_default(),
        work_center: r.get::<_, Option<String>>(10)?.unwrap_or_default(),
        test_frameid_no: r.get::<_, Option<String>>(11)?.unwrap_or_default(),
      })
    })
    .unwrap_or_else(|e| die(&format!("读取结果失败: {e}")));
  rows.filter_map(Result::ok).collect()
}

/// 由 prod 域名推导前端 API 根：http://heg-pc-cmesprod.gz.cvte.cn → http://heg-pc-cmes.gz.cvte.cn/frontApi/prod
fn front_url_from(prod_url: &str) -> String {
  if !prod_url.contains("-cmesprod.") {
    return String::new();
  }
  let base = prod_url.replace("-cmesprod.", "-cmes.");
  format!("{}/frontApi/prod", base.trim_end_matches('/'))
}

fn render(row: &Row, a: &Args) -> String {
  let mut lines: Vec<String> = vec![
    "@echo off".into(),
    "REM 由 cvte-mes-gencfg 从 etest.db 生成 —— 含明文凭据，勿提交/外发".into(),
    format!(
      "REM 来源：org_cvte_cmes.id={}  站点={}  企业={}",
      row.id, row.station_name, row.org_tag
    ),
    format!("set \"PROD_URL={}\"", row.prod_url),
    format!("set \"SYS_URL={}\"", row.sys_url),
    format!("set \"USER_NAME={}\"", row.username),
    format!("set \"PASS_WORD={}\"", row.password),
    format!("set \"TOKEN_NAME={}\"", row.token_name),
    format!("set \"CAUSE_NO={}\"", row.cause_no),
    format!("set \"WORK_CENTER={}\"", row.work_center),
    format!("set \"TEST_FRAME_ID={}\"", row.test_frameid_no),
    format!("set \"STATION_CODE={}\"", row.station_code),
    format!("set \"FRONT_URL={}\"", front_url_from(&row.prod_url)),
  ];
  for (k, v) in [
    ("SN", &a.sn),
    ("SN2", &a.sn2),
    ("LOT", &a.lot),
    ("LOT2", &a.lot2),
    ("PARENT_ID", &a.parent_id),
  ] {
    if let Some(v) = v {
      lines.push(format!("set \"{k}={v}\""));
    }
  }
  lines.join("\r\n") + "\r\n"
}

fn main() {
  let a = parse_args();
  let rows = load_rows(&a.db);

  if a.list {
    println!("etest.db: {}", a.db);
    for r in &rows {
      println!(
        "  id={:<3} [{}] {:<8} {:<40} {}",
        r.id,
        if r.usable() { "可用" } else { "不完整" },
        r.station_name,
        r.prod_url,
        r.username
      );
    }
    return;
  }

  let usable: Vec<&Row> = rows.iter().filter(|r| r.usable()).collect();
  if usable.is_empty() {
    die("org_cvte_cmes 里没有可用配置（prod_url / username 为空）");
  }
  let row = match a.id {
    Some(id) => match usable.iter().find(|r| r.id == id) {
      Some(r) => *r,
      None => die(&format!("没有 id={id} 的可用配置，用 --list 看候选")),
    },
    None => *usable.last().expect("非空已判断"),
  };

  if let Err(e) = fs::write(&a.out, render(row, &a)) {
    die(&format!("写入 {} 失败: {e}", a.out));
  }
  println!("[OK] 已生成 {}", a.out);
  println!(
    "     来源 id={}  站点={}  {}  账号={}",
    row.id, row.station_name, row.prod_url, row.username
  );
  println!("     该文件含明文密码，请勿外发；verify.bat 会自动加载它。");
}
