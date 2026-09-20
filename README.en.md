<div align="center">
  <h1>🧩 cvte-mes</h1>
  <p><strong>Unified MES interface library (CMES / SCM WIP / ItAPI)</strong></p>
  <p>Auth · SN station pass · Production reporting · Feed sheet · Component list · Pack finish qty · MAC binding · Software info</p>
</div>

<div align="center">
  📄 [MIT](LICENSE) · 📦 [crates.io](https://crates.io/crates/cvte-mes) · 📚 [docs.rs](https://docs.rs/cvte-mes) · 🌟 [Gitee](https://gitee.com/eternalnight996/cvte-mes) · 🧪 35 tests
</div>

<div align="center"><a href="README.md">简体中文</a> | English</div>

---

## ✨ Features

- 🔑 **Unified auth**: CMES login token, SCM MD5 signing, ItAPI apikey — each in its place
- 🔐 **CMES integration standard**: token cache (2 h / refresh 5 min early / one login for concurrent callers), automatic re-login on HTTP 401 or `unAuthorizedRequest` with **at most one retry**, layered responses for HTTP 500/401/200 (hint taken from `error.message` → `error.details`)
- 🔄 **Async-first + optional blocking**: native async; `blocking` feature for GUI callers (`CmesSession` keeps the token cached)
- 📦 **Feed sheet for box building**: real `GetFeedSheetList` endpoint (moLotNo accepts batch no. or SN)
- 🧩 **Pack endpoints**: `GetOtherListBySn` (SN → bound component list), `GetSnPackFinishQty` (batch → packed SN count, split by machine/part)
- 🖥 **CLI harness**: `cargo run -- --api CMES|SCM|ItAPI --task <Task>` to verify each endpoint on the command line; `scripts\verify.bat` runs all 13 endpoints and prints a colored PASS/FAIL table
- 🐧 **Cross-platform**: Windows / Linux (Kylin V10 / UOS V20 / Ubuntu 18.04+)

---

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

```rust
// Async: session reuse (cached token + one automatic retry after re-login)
use cvte_mes::mes::cmes::{session::Session, Cfg};
let cfg = Cfg {
  sysUrl: "http://xxx-cmessys.gz.cvte.cn".into(),      // login host
  prodUrl: "http://xxx-cmesprod.gz.cvte.cn".into(),
  userName: "u".into(),
  passWord: "p".into(),
  tokenName: "cmes_xxx_info".into(),                   // empty → falls back to the standard x-auth-token header
  ..Default::default()
};
let s = Session::new(cfg);
let front = "http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod";   // front-end API root
let comps = s.get_other_list_by_sn(front, "U23070429-0A03693").await?;    // SN → component list
let qty = s.get_sn_pack_finish_qty(front, "HEG-YKCBU2607034701").await?;  // batch → packed SN counts
println!("{} components, {} qty rows", comps.len(), qty.len());
```

```rust
// Blocking (feature "blocking"): build the session once in a GUI, log in only once
use cvte_mes::blocking::CmesSession;
let s = CmesSession::new(&cfg)?;
let comps = s.get_other_list_by_sn(front, "U23070429-0A03693")?;
```

**Tests (real output)**:

```sh
$ cargo test --lib
running 35 tests
test mes::cmes::feed_sheet::tests::item_from_json_picks_known_fields ... ok
test mes::cmes::pack::tests::parse_other_items_reads_documented_fields ... ok
test result: ok. 34 passed; 0 failed
```

---

## 📊 Endpoint overview

| System | Module | Endpoint (CLI task) | Description |
|--------|--------|---------------------|-------------|
| CMES | `mes::cmes` | `User:CreateToken` | Login (account with `@` uses MD5, otherwise Base64) |
| CMES | `mes::cmes` | `TestScan:CheckLotSnCode` | SN validation / station pass |
| CMES | `mes::cmes` | `TestScan:GetInfoBySn` | Batch info by SN |
| CMES | `mes::cmes` | `TestScan:SaveProduceTestByAutoTest` | Save test result (report) |
| CMES | `mes::cmes` | `TestScan:GetProduceDetails` | Query test item results |
| CMES | `mes::cmes` | `Record:InsertAtSpi` | Serial scan station pass |
| CMES | `mes::cmes` | `ScanHead:GetFeedSheetList` | Feed sheet query (moLotNo = batch/SN) |
| CMES | `mes::cmes::pack` | `Pack:GetOtherListBySn` | **Component list by product barcode** (2026-09 standard) |
| CMES | `mes::cmes::pack` | `Pack:GetSnPackFinishQty` | **Packed SN count by batch** (machine/part rows) |
| CMES | `mes::cmes::session` | function `Session` | Token cache + retry-on-401 + layered response parsing (shared) |
| SCM | `mes::scm::wip` | `SCM:MoFactory` | MO info (MD5 signed) |
| SCM | `mes::scm::wip` | `SCM:ReqCommon` | Feed sheet info (MD5 signed) |
| ItAPI | `mes::itapi` | `ItAPI:BatchProdDocs` | Batch CPLM production docs |
| ItAPI | `mes::itapi` | `ItAPI:SoftInfoListV2` | Software info table |
| Generic | `mes::mac` | function `query_macs_by_sn` | SN→MAC binding (anti-mix, CMES/ItAPI auth) |

---

## 🔐 CMES integration standard (2026-09 revision)

| Step | Standard | Implementation |
|------|----------|----------------|
| Get token | `GET {host}/api/Account/CreateToken?Email=&Pwd=` (Pwd = 32-char lowercase MD5 digest) | `login::login` |
| Send token | header `x-auth-token: {Token}` (header names are case-insensitive) | `session::auth_headers` (uses `Cfg::tokenName` when set, otherwise the standard name) |
| Token lifetime | 2 h by default; refresh before expiry; concurrent refreshes must be serialized | `session::Session` (refresh 5 min early, one login for concurrent callers) |
| Token invalid | HTTP 401 or `unAuthorizedRequest === true` → drop the old token, re-login, **retry the original request at most once** | `Session::get_json`; a failed re-login stops the retry and must alert |
| HTTP 500 | Server error or operator-friendly message: read `error.message` / `error.details` or the body text, **never parse `result`** | `session::classify` (hint is safe to show to operators) |
| HTTP 200 + `success !== true` | Business failure: read `error`, do not use `result` | same as above |
| HTTP 200 + `success === true` | Success: business data lives in `result` | `session::classify` |
| Other status / non-JSON | Uncontracted HTTP error / protocol error (log status + redacted raw response) | same as above |
| Parameter errors | blank `snCode` / `moLotNo` → no request is sent | pre-checks in `pack::get_other_list_by_sn` / `get_sn_pack_finish_qty` |
| Rate limit | `GetSnPackFinishQty` is throttled per `moLotNo` (default 30 calls / 10 s) | caller-side pacing; do not hot-loop |
| Security | account, password, MD5 digest and token must never reach logs or error messages; token travels in the header only | errors carry no credentials; grant the integration account least privilege |

---

## 🖥 CLI examples

```sh
# CMES login (test env)
cargo run -- --api CMES --task User:CreateToken --setting '{"stationName":"test","prodUrl":"http://url.cn","sysUrl":"http://url.cn","userName":"uname","passWord":"passwd","tokenName":"cmes_test_info","causeNo":"test","workCenter":"TEST01","testFrameidNo":"TEST","stationCode":"test"}'

# Component list by product barcode (--args[0]=front-end API root, --args[1]=barcode)
cargo run -- --api CMES --task Pack:GetOtherListBySn --args 'http://heg-pc-cmes.gz.cvte.cn/frontApi/prod' 'U23070429-0A03693' --setting '<CMES setting>'

# Packed SN count by batch (--args[0]=front-end API root, --args[1]=batch no.)
cargo run -- --api CMES --task Pack:GetSnPackFinishQty --args 'http://heg-pc-cmes.gz.cvte.cn/frontApi/prod' 'HEG-YKCBU2607034701' --setting '<CMES setting>'

# SCM MO query
cargo run -- --api SCM --task SCM:MoFactory --setting '{"gateway":"https://itgw-sit.cvte.com/env-101/scm/scmwip/gw-wip","apikey":"<appId>","api_secret":"<secret>"}' --args '{"lineLotNumber":"HEG-SKZBU23070075"}'

# ItAPI MAC query by SN
cargo run -- --api ItAPI --task ItAPI:MacQuery --setting '{"query_url":"https://.../GetMacBySn","station_code":"ZS220","apikey":"<appId>"}' --args 'SN12345678'
```

---

## 🧪 Batch verification script (scripts/)

Runs all 13 endpoints in one go and prints a colored PASS/FAIL table:

| File | Purpose |
| --- | --- |
| `scripts/verify.bat` | Generic template: fill the config block with your env / account / barcode |
| `scripts/verify2.bat` | On-site real-account variant (plaintext credentials, gitignored, **never commit**) |
| `scripts/_render.ps1` | Colored table renderer called by the bats, holds no credentials |

```bat
:: exe lookup order: scripts\cvte-mes.exe -> target\release -> target\debug
cargo build --release --features input-api,blocking
copy target\release\cvte-mes.exe scripts\
scripts\verify.bat
```

- Coverage: 8 CMES endpoints (including the two write endpoints `Record:InsertAtSpi` and `TestScan:SaveProduceTestByAutoTest`, skipped by default; set `RUN_WRITE=1` to run them) + 2 SCM + 3 ItAPI
- Output: per-endpoint `PASS` (green) / `FAIL` (red) / `ERROR` (yellow), plus a summary and failure details; raw responses stay in `scripts\verify.runs\`
- Exit code: `1` when anything failed, `0` when all green; the window stays open when finished
- The whole `scripts\` folder can be copied to an intranet machine and run by double-click (no project root needed)

---

### Built-in self-test (no production access)

Scenarios a script cannot construct — token cache, one login under concurrency, automatic re-login on 401 with **at most one retry**, layered parsing of HTTP 500 / business failure / non-JSON, and the blocking sync path — are covered by a built-in self-test: it starts a mock server on localhost and **never touches production or writes business data**.

```bat
cargo run --features input-api,blocking -- --api CMES --task SelfTest:Session
```

### Where the production config comes from

`scripts/gen_prod_cfg.py` reads accounts/hosts from the `org_cvte_cmes` table in `etest.db` and writes `verify.prod.bat` (plaintext credentials, gitignored); `verify.bat` loads it automatically when present, so switching account or factory needs no edit to the script itself:

```sh
python scripts/gen_prod_cfg.py --list                        # list candidates
python scripts/gen_prod_cfg.py --id 1 --sn <SN> --lot <LOT>  # generate
```

---

## 🐧 Linux build (Kylin V10 / UOS V20 / Ubuntu 18.04+)

```bash
cargo build --release --features input-api,blocking
```

> `input-api` is the feature the CLI needs (`src/main.rs` uses `cvte_mes::input`); building the bin without it fails with `could not find input in cvte_mes`.

- reqwest 0.13 + rustls/ring: pure-Rust TLS — no system OpenSSL or CMake required, for HTTPS and cross-compilation alike
- Cross-checking from Windows (`--target x86_64-unknown-linux-gnu`) works directly (no openssl-sys)
- rust-version >= 1.85 (reqwest 0.13 requirement)

---

## 🔧 Development

- Commit prefixes: `feat` / `fix` / `chore` / `docs` / `refactor` / `test`
- New endpoints: follow the existing module pattern (Cfg + request + parse + unit test); new CMES endpoints go through `session::get_json_with_token`; field names follow actual API responses
- Before commit: `cargo test` green + `cargo clippy --all-features --all-targets` with no new warnings

---

## 📋 Changelog

| Version | Date | Highlights |
|---|---|---|
| **v0.2.0-rc.1** | 2026-09-20 | **CMES integration standard completed**: new `pack` module — `GetOtherListBySn` (SN → component list) and `GetSnPackFinishQty` (batch → packed SN counts, split by machine/part); new `session` module — token cache (2 h, refresh 5 min early, one login for concurrent callers), automatic re-login on HTTP 401 / `unAuthorizedRequest` with **at most one retry**, and layered parsing for HTTP 500/401/200/uncontracted/non-JSON (hint from `error.message` → `error.details`); two new `Pack:*` CLI tasks; blocking `CmesSession`; tests 16 → 34; new `scripts/` batch verification (13 endpoints, colored PASS/FAIL table); fixed the SCM/ItAPI panic caused by a missing rustls provider |
| v0.1.2 | 2026-08-28 | High-level `cmes::mo_lot_no` / `cmes::report` chains + blocking wrappers |
| v0.1.1 | 2026-08-28 | reqwest switched to rustls + ring (keeps HTTPS, no system OpenSSL) |
| v0.1.0 | 2026-08-14 | First release: CMES (login/scan/report/feed sheet) + SCM WIP (MD5 signed) + ItAPI (production docs/software info) + MAC binding query |

---

## 📄 License

[MIT](LICENSE)

<div align="center"><sub>Built with ❤️ by EternalNight</sub></div>
