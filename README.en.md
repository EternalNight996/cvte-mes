<div align="center">
  <h1>🧩 cvte-mes</h1>
  <p><strong>Unified MES interface library (CMES / SCM WIP / ItAPI / MAC)</strong></p>
  <p>Auth · SN station pass · Production reporting · Feed sheet · Component list · Pack finish qty · MAC binding · Software info · Front-end API auth diagnose</p>
</div>

<div align="center">

📄 [MIT](LICENSE) · 📦 [crates.io](https://crates.io/crates/cvte-mes) · 📚 [docs.rs](https://docs.rs/cvte-mes) · 🌟 [Gitee](https://gitee.com/eternalnight996/cvte-mes) · 🐙 [GitHub](https://github.com/eternalnight996/cvte-mes) · 🧪 48 tests · 🚀 v0.2.0-rc.2

</div>

<div align="center"><a href="README.md">简体中文</a> | English</div>

---

## ✨ Features

- 🔑 **Unified auth**: CMES login token, SCM MD5 gateway signing, ItAPI apikey — each in its place
- 🔐 **CMES integration standard**: token cache (2 h / refresh 5 min early / one login for concurrent callers); automatic re-login on HTTP 401 or `unAuthorizedRequest === true` with **at most one retry**; layered parsing for HTTP 500 / business failure / protocol errors (hint taken from `error.message` → `error.details` → raw body)
- 🔄 **Async-first + optional blocking**: native async; `blocking` feature for GUI callers (`CmesSession` keeps the token cached)
- 📦 **Feed sheet for box building**: real `GetFeedSheetList` endpoint (moLotNo accepts batch no. or SN)
- 🧩 **Pack endpoints**: `GetOtherListBySn` (SN → bound component list), `GetSnPackFinishQty` (batch → packed SN count, split by machine/part); blank inputs rejected before any HTTP call
- 🏭 **Base-data (bas) endpoints** (standard V1.1 chapters 10–12): `GetFeedSheetByContainerName` (barcode → feed-sheet material rows of its batch, need-qty > 0, sorted by part no. desc), `GetStationPassQty` (batch → per-station pass counts, split by machine/part, deduplicated by barcode), `GetInfoBySn` (barcode detail); **a missing barcode yields `Ok(None)` instead of an error**; pre-checks for `moLotNo` ≤ 100 chars and `stationCodes` ≤ 50 entries
- 🛡 **Front-end API auth diagnose**: `Diagnose:Auth` probes the same URL five times with the same Token but different header names (`x-auth-token` / `Authorization: Bearer` / `tokenName` / `Cookie` / `token`) and reports the HTTP status for each so you can see which auth scheme the front-end accepts (Token value is never logged)
- 🧪 **Built-in self-test (`SelfTest:Session`)**: a local mock server answers with the documented payloads and validates the session layer — token cache, one login under concurrency, automatic re-login on 401 with **at most one retry**, HTTP 500 / business failure / non-JSON parsing, plus parsing of the three new V1.1 ch.10–12 endpoints — **15 checks** in total. **No production access, no business side effects.**
- 🖥 **CLI harness**: `cargo run -- --api CMES|SCM|ItAPI --task <Task>` to verify each endpoint on the command line; `scripts\verify.bat` runs all 28 read-only cases (main account 14 + board group 9 + SCM 2 + ItAPI 3; `RUN_WRITE=1` adds 3 write cases) and prints a colored PASS/FAIL table
- 🐧 **Cross-platform**: Windows / Linux (Kylin V10 / UOS V20 / Ubuntu 18.04+); reqwest 0.13 + rustls/ring (pure-Rust TLS, no system OpenSSL/CMake required)

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

```rust
// Base data (standard V1.1, chapters 10–12)
let parts = s.feed_sheet_by_container_name(front, "CIS-CH11HG260904940XAG0010").await?; // 10: barcode → feed-sheet rows
let stops = s.station_pass_qty(front, "HEG-YKCBU2607034701", Some("B10,B20")).await?;  // 11: batch → per-station pass counts
match s.serial_no_info(front, "CIS-CH11HG260904940XAG0010", None, None).await? {       // 12: barcode detail
  Some(info) => println!("barcode {} batch {}, capacity {}", info.sn, info.moLotNo, info.snQty),
  // When the barcode does not exist the server returns result: null with success: true —
  // that is "not found", not an error.
  None => println!("barcode not found"),
}
```

**Tests (real output)**:

```sh
$ cargo test --lib
running 48 tests
test mes::cmes::bas::tests::parse_serial_info_null_means_barcode_not_found ... ok
test mes::cmes::bas::tests::session_serial_no_info_maps_null_result_to_none ... ok
test mes::cmes::pack::tests::parse_other_items_reads_documented_fields ... ok
test selftest::tests::selftest_checks_pass ... ok
test result: ok. 48 passed; 0 failed
```

---

## 📊 Endpoint overview (22 CLI tasks)

### CMES (17)

| Module | `--task` | Description |
|--------|----------|-------------|
| `cmes::login` | `User:CreateToken` | Login (account with `@` uses MD5, otherwise Base64) |
| `cmes::test_scan` | `TestScan:CheckLotSnCode` | SN validation / station pass |
| `cmes::test_scan` | `TestScan:GetInfoBySn` | Batch info by SN |
| `cmes::test_scan` | `TestScan:GetProduceDetails` | Test item results (**GET**, 411 fixed) |
| `cmes::test_scan` | `TestScan:SaveProduceTestByAutoTest` | Save test result (report) |
| `cmes::record` | `Record:InsertAtSpi` | Serial scan station pass |
| `cmes::feed_sheet` | `FeedSheet:GetFeedSheetList` | Feed sheet query (moLotNo = batch/SN) |
| `cmes::pack` | `Pack:GetOtherListBySn` | **Component list by product barcode** (2026-09 standard) |
| `cmes::pack` | `Pack:GetSnPackFinishQty` | **Packed SN count by batch** (machine/part rows) |
| `cmes::bas` | `FeedSheetOrder:GetFeedSheetByContainerName` | **V1.1 ch.10**: barcode → feed-sheet material rows of its batch (need-qty > 0, sorted by part no. desc) |
| `cmes::bas` | `ScanHistory:GetStationPassQty` | **V1.1 ch.11**: batch → per-station pass counts (machine/part split, deduped by barcode; `moLotNo` ≤ 100 chars, `stationCodes` ≤ 50 entries) |
| `cmes::bas` | `SerialNoCache:GetInfoBySn` | **V1.1 ch.12**: barcode detail; **missing barcode returns `result: null` and is not an error** (library returns `Option::None`) |
| `cmes` | `CMES:MoLotNo` | High-level: SN → batch (login+validate chain) |
| `cmes` | `CMES:SessionToken` | Return the Session-cached Token (proves the session layer) |
| `cmes` | `CMES:Report` | High-level: SN → report (login+validate+report chain) |
| `cmes::session` | `SelfTest:Session` | **Built-in self-test**: local mock validates token cache / retry / layered parsing / blocking — **no production access** |
| `cmes::login` | `Diagnose:Auth <url>` | **Front-end API auth diagnose**: same Token, five header candidates, compare status codes to learn which auth scheme the front-end accepts (Token value never logged) |

### SCM WIP (2, MD5 gateway signing)

| Module | `--task` | Description |
|--------|----------|-------------|
| `scm::wip` | `SCM:MoFactory` | MO info (**aligned with the official `DefaultSignSDK`**: only String participates in signing + struct null tolerance + raw response on error) |
| `scm::wip` | `SCM:ReqCommon` | Feed sheet info (same signing convention) |

### ItAPI (3)

| Module | `--task` | Description |
|--------|----------|-------------|
| `itapi` | `ItAPI:BatchProdDocs` | Batch CPLM production files |
| `itapi` | `ItAPI:SoftInfoListV2` | Software info table |
| `itapi` / `mac` | `ItAPI:MacQuery` | MAC binding by SN |

### Library functions (call anywhere)

| Module | Function | Description |
|--------|----------|-------------|
| `mac` | `query_macs_by_sn` | SN → MAC binding (anti-mix, CMES/ItAPI auth compatible) |

---

## 🔐 CMES integration standard (V1.1 / 2026-10-14)

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
| Parameter errors | blank `snCode` / `moLotNo` / `containerName` / `sn` → no request is sent; `moLotNo` > 100 chars and `stationCodes` > 50 entries are rejected too | pre-checks in `pack` / `bas` (`check_sn` / `check_lot` / `check_container` / `check_station_codes`) |
| "Not found" ≠ error | `bas/SerialNoCache/GetInfoBySn`: when the barcode does not exist, **`success` is still `true` and `result` is `null`** — judge by `result`, **never parse `error`** | `bas::serial_no_info` → `Ok(None)`; pinned by `parse_serial_info` plus an end-to-end test |
| null tolerance | in ch.12 with `isIncludeOrder=false` several fields come back as `null` (the doc's per-field annotation is incomplete — measured: the order fields `orderId`/`orderNo`/`moLotNo` are `null` too) | `bas::without_nulls` treats `null` ≡ missing ≡ default so a single `null` cannot fail a whole row |
| Rate limit | `GetSnPackFinishQty` / `GetFeedSheetByContainerName` / `GetStationPassQty` are throttled per `moLotNo` / `containerName` / `moLotNo` (default 30 calls / 10 s); `GetInfoBySn` has no limit | caller-side pacing; do not hot-loop |
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

# [V1.1-10] Feed-sheet material rows by barcode (--args[0]=front-end API root, --args[1]=barcode)
cargo run -- --api CMES --task FeedSheetOrder:GetFeedSheetByContainerName --args 'http://heg-pc-cmes.gz.cvte.cn/frontApi/prod' 'CIS-CH11HG260904940XAG0010' --setting '<CMES setting>'

# [V1.1-11] Per-station pass counts by batch (--args[2]=station codes, optional, comma-separated, max 50)
cargo run -- --api CMES --task ScanHistory:GetStationPassQty --args 'http://heg-pc-cmes.gz.cvte.cn/frontApi/prod' 'HEG-YKCBU2607034701' 'B10,B20' --setting '<CMES setting>'

# [V1.1-12] Barcode detail (--args[2]=isIncludeOrder optional; --args[3]=componentBasisCode optional)
#   Note: for a missing barcode the output has content: null while status stays true (not an error)
cargo run -- --api CMES --task SerialNoCache:GetInfoBySn --args 'http://heg-pc-cmes.gz.cvte.cn/frontApi/prod' 'CIS-CH11HG260904940XAG0010' --setting '<CMES setting>'

# Front-end API auth diagnose: probe the same URL with five header candidates using one Token
cargo run -- --api CMES --task Diagnose:Auth --args 'http://heg-pc-cmes.gz.cvte.cn/frontApi/prod/api/services/pack/Pack/GetOtherListBySn?snCode=U23070429-0A03693' --setting '<CMES setting>'

# Built-in self-test (local mock, zero external dependencies)
cargo run --features input-api,blocking -- --api CMES --task SelfTest:Session

# SCM MO query (MD5 signing, only String participates)
cargo run -- --api SCM --task SCM:MoFactory --setting '{"gateway":"https://itgw-sit.cvte.com/env-101/scm/scmwip/gw-wip","apikey":"<appId>","api_secret":"<secret>"}' --args '{"lineLotNumber":"HEG-SKZBU23070075"}'

# ItAPI MAC query by SN
cargo run -- --api ItAPI --task ItAPI:MacQuery --setting '{"query_url":"https://.../GetMacBySn","station_code":"ZS220","apikey":"<appId>"}' --args 'SN12345678'
```

---

## 🧪 Batch verification script (scripts/)

Runs all 28 read-only cases in one go (main account 14 + board group 9 + SCM 2 + ItAPI 3; `RUN_WRITE=1` adds 3 write cases) and prints a colored PASS/FAIL table:

| File | Purpose | Credentials |
| --- | --- | --- |
| `scripts/verify.bat` | Generic template; loads it or auto-generates from `etest.db` when `cvte-mes-gencfg.exe` is present | none (external) |
| `scripts/verify.prod.bat` | **Production config**: account / host / barcodes; generated by `cvte-mes-gencfg.exe` | ✅ contains credentials, gitignored |
| `scripts/verify.prod.board.bat` | **Board-group config**: a separate site/account for board barcodes (which belong to a different site/organization) | ✅ contains credentials, gitignored |
| `scripts/gen-cfg.bat` | List candidates from `etest.db` → pick an id → write `verify.prod.bat` | none |
| `scripts/_render.ps1` | Colored table renderer (called by the bats) | none |
| `scripts/cvte-mes-gencfg.exe` | Static SQLite reader (**bundled**, no Python needed on site) | — |
| `scripts/cvte-mes.exe` | CLI main binary (copied from `target\release`) | — |

```bat
:: exe lookup order: scripts\cvte-mes.exe -> target\release -> target\debug
cargo build --release --features input-api,blocking
copy target\release\cvte-mes.exe scripts\
scripts\verify.bat
```

- **Coverage**: 14 main-account cases (`SelfTest:Session`, `User:CreateToken`, 6 read-only queries, the three V1.1 additions `FeedSheetOrder`/`ScanHistory`/`SerialNoCache`, `CMES:MoLotNo`, `CMES:SessionToken`, `Diagnose:Auth`) + 9 board-group cases (same set against the second account/site) + 2 SCM + 3 ItAPI = **28**; the three write endpoints (`Record:InsertAtSpi`, `TestScan:SaveProduceTestByAutoTest`, `CMES:Report`) are skipped unless `RUN_WRITE=1` (31 total)
- **Skip conditions**: the main group needs `SN` (blank skips that block; `CMES:SessionToken` / `Diagnose:Auth` still run); the board group needs `SN2`
- **Production config auto-load**: `verify.bat` calls `verify.prod.bat` when present; if missing and `cvte-mes-gencfg.exe` + `etest.db` sit next to it, the config is regenerated; missing main-config keys fall back inside the script (e.g. default `PARENT_ID`)
- **Board-group dual account**: any blank key in `verify.prod.board.bat` falls back to the main config; board barcodes (e.g. `B26082334-*`) require the dedicated board credentials
- **Host fallback**: `PROD_URL2` / `SYS_URL2` / `FRONT_URL2` fall back to `PROD_URL` / `SYS_URL` / `FRONT_URL`
- **Output**: per-case `PASS` (green) / `FAIL` (red) / `ERROR` (yellow), plus a summary and failure details; raw responses stay in `scripts\verify.runs\`; each run is archived under `logs\<N>\` (result.txt + full runs/)
- **Exit code**: `1` when anything failed, `0` when all green; the window stays open when finished
- **Self-contained**: the whole `scripts\` folder can be copied to an intranet machine and run by double-click (no project root needed)

### Built-in self-test (no production access)

Scenarios a script cannot construct — token cache, one login under concurrency, automatic re-login on 401 with **at most one retry**, layered parsing of HTTP 500 / business failure / non-JSON, the blocking sync path, plus parsing of the three new V1.1 ch.10–12 endpoints (including the "missing barcode → `Ok(None)`" convention) — are covered by a built-in self-test: it starts a mock server on localhost that answers with the documented payloads, and **never touches production or writes business data**.

```bat
cargo run --features input-api,blocking -- --api CMES --task SelfTest:Session
```

Fifteen checks (any failure makes `cargo test` red):

| # | Check | Maps to |
|---|-------|---------|
| 1 | Back-to-back token reads → one login, same value | §6 · token cache |
| 2 | 4 concurrent token reads → still one login | §6 · one login under concurrency |
| 3 | HTTP 401 → re-login and retry once → success | §4.3 / §5.2 |
| 4 | Persistent 401 → stop after one retry (no retry storm) | §5.2 |
| 5 | Explicit-Token path flags 401 as "token invalid" | §4.3 |
| 6 | HTTP 500 reads the server message (not `result`) | §4.3 order 1 |
| 7 | HTTP 200 + success=false → use `error.message` | §4.3 order 3 |
| 8 | HTTP 200 but non-JSON → protocol error | §4.3 order 6 |
| 9 | Plain HTTP 200 success reads `result` | §4.3 order 4 |
| 10 | Ch.10: barcode → feed-sheet material rows parsed correctly (bool / decimals) | V1.1 §10.3 |
| 11 | Ch.11: per-station rows parsed correctly, `stationCode: null` not misjudged | V1.1 §11.4 |
| 12 | Ch.12: barcode detail object parsed correctly | V1.1 §12.3 |
| 13 | Ch.12: **missing barcode (`result: null`) returns `Ok(None)`, not an error** | V1.1 §12.2 |
| 14 | Blank `containerName` rejected up-front with no request sent | V1.1 §10.4 |
| 15 | Blocking sync path works (requires the `blocking` feature) | GUI scenarios |

### Where the production config comes from (no Python needed on site)

`cvte-mes-gencfg.exe` reads accounts/hosts from the `org_cvte_cmes` table in `etest.db` and writes `verify.prod.bat` (plaintext credentials, gitignored); `verify.bat` loads it automatically when present, so switching account or factory needs no edit to the script itself. SQLite is **statically compiled** into that exe — no Python or other external dependency on site.

```bat
:: only when the config must be regenerated (build once, ship the exe)
cargo build --release --features gen-cfg

:: on site, double-click gen-cfg.bat: list candidates -> pick an id -> write verify.prod.bat
gen-cfg.bat
gen-cfg.bat D:\path\to\etest.db     :: or pass an explicit etest.db path
```

A Python variant remains available on dev machines:

```sh
python scripts/gen_prod_cfg.py --list
python scripts/gen_prod_cfg.py --id 1 --sn <SN> --lot <LOT>
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

## 📁 Project layout

```
cvte-mes/
├── src/
│   ├── mes/
│   │   ├── cmes/        # CMES: login / test_scan / record / feed_sheet / pack / bas / session
│   │   ├── scm/         # SCM WIP: wip + sign (MD5 gateway signing)
│   │   ├── itapi/       # ItAPI: CPLM production docs / software info table
│   │   └── mac.rs       # SN→MAC binding (CMES / ItAPI auth compatible)
│   ├── input/api.rs     # CLI task dispatch
│   ├── blocking.rs      # Sync wrappers (feature = "blocking")
│   ├── selftest.rs      # Built-in self-test (local mock)
│   └── bin/gen_cfg.rs   # cvte-mes-gencfg (feature = "gen-cfg")
├── scripts/             # Batch verification scripts (see §"Batch verification script")
├── doc/                 # Archived partner docs (CMES / SCM WIP / ItAPI / official signing demo)
└── Cargo.toml
```

---

## 🔧 Development

- Commit prefixes: `feat` / `fix` / `chore` / `docs` / `refactor` / `test`
- New endpoints: follow the existing module pattern (Cfg + request + parse + unit test); new CMES endpoints go through `session::get_json_with_token`; field names follow actual API responses
- SCM signing follows the official `DefaultSignSDK`: only String participates; struct `null` fields tolerated; errors carry the raw response
- V1.1 `bas` parsing runs every row through `without_nulls`: `#[serde(default)]` covers a *missing* field but not a field that is explicitly `null`
- Before commit: `cargo test` green (48/48) + `cargo clippy --all-features --all-targets` with no new warnings

---

## 📋 Changelog

| Version | Date | Highlights |
|---|---|---|
| **v0.2.0-rc.2** | 2026-10-16 | **CMES integration standard V1.1 (2026-10-14), chapters 10–12 landed**: new `cmes::bas` module — `GetFeedSheetByContainerName` (barcode → feed-sheet material rows of its batch, need-qty > 0, sorted by part no. desc), `GetStationPassQty` (batch → per-station pass counts, machine/part split, deduplicated by barcode, `stationCode` may be `null`), `GetInfoBySn` (barcode detail; **missing barcode returns `result: null` and is not an error — the library returns `Ok(None)`**); all three expose `Session` and blocking `CmesSession` variants; three new CLI tasks `FeedSheetOrder:*` / `ScanHistory:*` / `SerialNoCache:*`; pre-checks for blank `containerName`/`sn`, `moLotNo` ≤ 100 chars, `stationCodes` ≤ 50 entries (plus `B10, B20` → `B10,B20` normalization); new `without_nulls` helper (`null` ≡ missing ≡ default) fixing a whole-row parse failure when `isIncludeOrder=false` returns `null` for the order fields; **built-in self-test `SelfTest:Session` grown from 10 to 15 checks** (4 new for the ch.10–12 payloads and the "missing barcode → `Ok(None)`" convention, 1 for blank-parameter rejection); tests 35 → 48; verification script 19 → 28 read-only cases (+3 write cases); fixed `verify.prod.board.bat` line endings (LF → CRLF, cmd mis-parsed it otherwise) |
| v0.2.0-rc.1 | 2026-09-20 | **CMES integration standard completed**: new `pack` module — `GetOtherListBySn` (SN → component list) and `GetSnPackFinishQty` (batch → packed SN counts, split by machine/part); new `session` module — token cache (2 h, refresh 5 min early, one login for concurrent callers), automatic re-login on HTTP 401 / `unAuthorizedRequest` with **at most one retry**, and layered parsing for HTTP 500/401/200/uncontracted/non-JSON (hint from `error.message` → `error.details`); two new `Pack:*` CLI tasks; blocking `CmesSession`; tests 16 → 35; new `scripts/` batch verification (colored PASS/FAIL table, main + board + self-test + auth diagnose); new built-in self-test `SelfTest:Session` (local mock, 10 checks); new front-end API auth diagnose `Diagnose:Auth` (5 header candidates); fixed the SCM/ItAPI panic caused by a missing rustls provider; SCM signing aligned with the official `DefaultSignSDK` (String-only + null tolerance + raw response on error); `GetProduceDetails` switched to GET to fix 411 |
| v0.1.2 | 2026-08-28 | High-level `cmes::mo_lot_no` / `cmes::report` chains + blocking wrappers |
| v0.1.1 | 2026-08-28 | reqwest switched to rustls + ring (keeps HTTPS, no system OpenSSL) |
| v0.1.0 | 2026-08-14 | First release: CMES (login/scan/report/feed sheet) + SCM WIP (MD5 signed) + ItAPI (production docs/software info) + MAC binding query |

---

## 📄 License

[MIT](LICENSE)

<div align="center"><sub>Built with ❤️ by EternalNight · mirrored on Gitee + GitHub</sub></div>