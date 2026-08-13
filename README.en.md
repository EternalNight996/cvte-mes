<div align="center">
  <h1>🧩 cvte-mes</h1>
  <p><strong>Unified MES interface library (CMES / SCM WIP / ItAPI)</strong></p>
  <p>Auth · SN station pass · Production reporting · Feed sheet query · MAC binding · Software info</p>
</div>

<div align="center">
  📄 [MIT](LICENSE) · 📦 [crates.io](https://crates.io/crates/cvte-mes) · 📚 [docs.rs](https://docs.rs/cvte-mes) · 🌟 [Gitee](https://gitee.com/eternalnight996/cvte-mes) · 🧪 16 tests
</div>

<div align="center"><a href="README.md">简体中文</a> | English</div>

## ✨ Features

- 🔑 **Unified auth**: CMES login token, SCM MD5 signing, ItAPI apikey — each in its place
- 🔄 **Async-first + optional blocking**: native async; `blocking` feature for GUI callers
- 📦 **Feed sheet for box building**: real `GetFeedSheetList` endpoint (moLotNo accepts batch no. or SN), used by PackPrint to derive boxes-per-carton
- 🖥 **CLI harness**: `cargo run -- --api CMES|SCM|ItAPI --task <Task>` to verify each endpoint on the command line
- 🐧 **Cross-platform**: Windows / Linux (Kylin V10 / UOS V20 / Ubuntu 18.04+), native-tls over system OpenSSL

## 🚀 Quick Start

```toml
# Cargo.toml
cvte-mes = { git = "https://gitee.com/eternalnight996/cvte-mes.git" }
# Add "blocking" feature for synchronous GUI callers
cvte-mes = { git = "https://gitee.com/eternalnight996/cvte-mes.git", features = ["blocking"] }
```

```rust
// Async: feed sheet query (login → GetFeedSheetList)
use cvte_mes::mes::cmes::{feed_sheet, login, Cfg};
let cfg = Cfg { sysUrl: "http://xxx-cmessys.gz.cvte.cn".into(), userName: "u".into(), passWord: "p".into(), ..Default::default() };
let token = login::login(&cfg).await?;
let items = feed_sheet::feed_sheet_list(&cfg, &token, "http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod", "<parentId>", "HEG-YKCBU2607034701").await?;
println!("{} feed sheet rows (multi-key field extraction)", items.len());
```

**Tests (real output)**:

```sh
$ cargo test --lib
test mes::cmes::feed_sheet::tests::item_from_json_picks_known_fields ... ok
test mes::mac::tests::norm_mac_ignores_separators_and_case ... ok
test result: ok. 16 passed; 0 failed
```

## 📊 Endpoint overview

| System | Module | Endpoint (CLI task) | Description |
|--------|--------|---------------------|-------------|
| CMES | `mes::cmes` | `User:CreateToken` | Login |
| CMES | `mes::cmes` | `TestScan:CheckLotSnCode` | SN validation / station pass |
| CMES | `mes::cmes` | `TestScan:GetInfoBySn` | Batch info by SN |
| CMES | `mes::cmes` | `TestScan:SaveProduceTestByAutoTest` | Save test result (report) |
| CMES | `mes::cmes` | `TestScan:GetProduceDetails` | Query test item results |
| CMES | `mes::cmes` | `Record:InsertAtSpi` | Serial scan station pass |
| CMES | `mes::cmes` | `ScanHead:GetFeedSheetList` | **Feed sheet query** (real endpoint, moLotNo = batch/SN) |
| SCM | `mes::scm::wip` | `SCM:MoFactory` | MO info (MD5 signed) |
| SCM | `mes::scm::wip` | `SCM:ReqCommon` | Feed sheet info (MD5 signed) |
| ItAPI | `mes::itapi` | `ItAPI:BatchProdDocs` | Batch CPLM production docs |
| ItAPI | `mes::itapi` | `ItAPI:SoftInfoListV2` | Software info table |
| Generic | `mes::mac` | `query_macs_by_sn` | SN→MAC binding (anti-mix, CMES/ItAPI auth) |

## 🖥 CLI examples

```sh
# CMES login (test env)
cargo run -- --api CMES --task User:CreateToken --setting '{"stationName":"test","prodUrl":"http://url.cn","sysUrl":"http://url.cn","userName":"uname","passWord":"passwd","tokenName":"cmes_test_info","causeNo":"test","workCenter":"TEST01","testFrameidNo":"TEST","stationCode":"test"}'

# SCM MO query
cargo run -- --api SCM --task SCM:MoFactory --setting '{"gateway":"https://itgw-sit.cvte.com/env-101/scm/scmwip/gw-wip","apikey":"<appId>","api_secret":"<secret>"}' --args '{"lineLotNumber":"HEG-SKZBU23070075"}'

# ItAPI MAC query by SN
cargo run -- --api ItAPI --task ItAPI:MacQuery --setting '{"query_url":"https://.../GetMacBySn","station_code":"ZS220","apikey":"<appId>"}' --args 'SN12345678'
```

## 🐧 Linux build (Kylin V10 / UOS V20 / Ubuntu 18.04+)

```bash
sudo apt install -y libssl-dev pkg-config   # native-tls uses system OpenSSL
cargo build --release
```

- reqwest 0.13 + native-tls: HTTPS via system openssl (Ubuntu 18.04 ships 1.1), avoids rustls-aws-lc CMake>=3.12 requirement
- Cross-checking from Windows (`--target x86_64-unknown-linux-gnu`) stops at openssl-sys (cannot locate Linux system libs) — expected; build on a real Linux machine
- rust-version >= 1.85 (reqwest 0.13 requirement)

## 🔧 Development

- Commit prefixes: `feat` / `fix` / `chore` / `docs` / `refactor` / `test`
- New endpoints: follow existing module pattern (Cfg + request + parse + unit test); field names follow actual API responses (multi-key compatible)
- Before commit: `cargo test` green + `cargo clippy` no new warnings

## 📄 License

[MIT](LICENSE)

<div align="center"><sub>Built with ❤️ by EternalNight</sub></div>
