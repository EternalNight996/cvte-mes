<div align="center">
  <h1>🧩 cvte-mes</h1>
  <p><strong>源控 MES 系统接口库（CMES / SCM WIP / ItAPI / MAC 统一封装）</strong></p>
  <p>登录鉴权 · SN 过站 · 报工回传 · 投料单查询 · 组件关联 · 包装完工数 · MAC 绑定 · 软件信息表 · 前端 API 鉴权诊断</p>
</div>

<div align="center">

📄 [MIT](LICENSE) · 📦 [crates.io](https://crates.io/crates/cvte-mes) · 📚 [docs.rs](https://docs.rs/cvte-mes) · 🌟 [Gitee](https://gitee.com/eternalnight996/cvte-mes) · 🐙 [GitHub](https://github.com/eternalnight996/cvte-mes) · 🧪 48 tests · 🚀 v0.2.0-rc.2

</div>

<div align="center">简体中文 | <a href="README.en.md">English</a></div>

---

## ✨ Features

- 🔑 **统一鉴权**：CMES 登录 Token、SCM MD5 网关签名、ItAPI apikey，三套体系各归其位
- 🔐 **CMES 对接标准**：Token 缓存（2 小时 / 到期前 5 分钟刷新 / 并发只登录一次）；HTTP 401 与 `unAuthorizedRequest === true` 自动重取并**最多重试一次**；HTTP 500 / 业务失败 / 协议异常分层解析（提示优先 `error.message` → `error.details` → 纯文本兜底）
- 🔄 **全异步 + 可选同步**：async 原生支持；GUI 场景启用 `blocking` feature 免运行时（`CmesSession` 会话复用）
- 📦 **投料单建箱**：`GetFeedSheetList` 真实接口（moLotNo 传批次号或 SN）
- 🧩 **包装（Pack）接口**：`GetOtherListBySn`（SN → 组件关联列表）、`GetSnPackFinishQty`（批次 → 包装完工数，整机/部件分行）；空参数前置校验不发请求
- 🏭 **基础数据（bas）接口**（《CMES 对外接口对接标准 V1.1》第 10~12 章）：`GetFeedSheetByContainerName`（条码 → 所属批次投料单物料行，应发数 > 0、按物料编码倒序）、`GetStationPassQty`（批次 → 各工步过站数，整机/部件分行、按条码去重；不传 `stationCodes` 返回**全部工站**）、`GetInfoBySn`（条码详细信息）；**条码不存在时 `GetInfoBySn` 返回 `Ok(None)` 而非报错**；`moLotNo` ≤ 100 位、`stationCodes` ≤ 50 个前置校验
- 🔁 **过站数闭环验收**：`ScanHistory:VerifyStationPassQty` 先取该批次**全部工站**，再**指定工站**复查并逐项比对（只返回指定工站 / 过站数与全量一致 / 行数一致 / 抽查时须收窄）；结论非 PASS 时进程失败，验证表格直接显示 FAIL
- 🛡 **前端 API 鉴权诊断**：`Diagnose:Auth` 用同一 Token 依次试 `x-auth-token` / `Authorization: Bearer` / `tokenName` / `Cookie` / `token` 五种头访问同一 URL，对比 HTTP 状态码定位认哪种鉴权（Token 值不落日志）
- 🧪 **内置自检（SelfTest:Session）**：本地 mock 服务端按文档真实报文应答，验证「并发只登录一次」「401 自动重取一次」「持续 401 不无限重试」「HTTP 500 取服务端提示」「业务失败读 error.message」「非 JSON 协议异常」以及 V1.1 第 10~12 章三接口解析——共 **16 项**，**不访问生产、不产生业务副作用**
- 🖥 **CLI 联调工具**：`cargo run -- --api CMES|SCM|ItAPI --task <Task>` 命令行验证每个接口；`scripts\verify.bat` 批量跑全部 31 个只读用例（主账号 16 + 板卡组 9 + SCM 2 + ItAPI 3；`RUN_WRITE=1` 再追加 3 个写入用例）并输出彩色 PASS/FAIL 表格
- 🐧 **跨平台**：Windows / Linux（Kylin V10 / UOS V20 / Ubuntu 18.04+）；reqwest 0.13 + rustls/ring（纯 Rust TLS，免系统 OpenSSL/CMake）

---

## 🚀 Quick Start

```toml
# Cargo.toml
cvte-mes = { git = "https://gitee.com/eternalnight996/cvte-mes.git" }
# GUI 同步调用场景加 blocking feature
cvte-mes = { git = "https://gitee.com/eternalnight996/cvte-mes.git", features = ["blocking"] }
```

```rust
// 异步：投料单查询（登录 → GetFeedSheetList）
use cvte_mes::mes::cmes::{feed_sheet, login, Cfg};
let cfg = Cfg { sysUrl: "http://xxx-cmessys.gz.cvte.cn".into(), userName: "u".into(), passWord: "p".into(), ..Default::default() };
let token = login::login(&cfg).await?;
let items = feed_sheet::feed_sheet_list(&cfg, &token, "http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod", "<parentId>", "HEG-YKCBU2607034701").await?;
println!("投料单 {} 条（字段多候选兼容，见 docs）", items.len());
```

```rust
// 异步：会话复用（Token 缓存 + 失效自动重取并最多重试一次）
use cvte_mes::mes::cmes::{session::Session, Cfg};
let cfg = Cfg {
  sysUrl: "http://xxx-cmessys.gz.cvte.cn".into(),      // 登录域名
  prodUrl: "http://xxx-cmesprod.gz.cvte.cn".into(),
  userName: "u".into(),
  passWord: "p".into(),
  tokenName: "cmes_xxx_info".into(),                   // 留空则回退对接标准头 x-auth-token
  ..Default::default()
};
let s = Session::new(cfg);
let front = "http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod";   // 前端 API 根
let comps = s.get_other_list_by_sn(front, "U23070429-0A03693").await?;    // SN → 组件关联列表
let qty = s.get_sn_pack_finish_qty(front, "HEG-YKCBU2607034701").await?;  // 批次 → 包装完工数
println!("组件 {} 条，完工 {} 行", comps.len(), qty.len());
```

```rust
// 同步（blocking feature）：GUI 场景一次建会话，反复查询只登录一次
use cvte_mes::blocking::CmesSession;
let s = CmesSession::new(&cfg)?;
let comps = s.get_other_list_by_sn(front, "U23070429-0A03693")?;
```

```rust
// 基础数据（《对接标准 V1.1》第 10~12 章）
let parts = s.feed_sheet_by_container_name(front, "CIS-CH11HG260904940XAG0010").await?; // 10：条码 → 投料单物料行
let stops = s.station_pass_qty(front, "HEG-YKCBU2607034701", Some("B10,B20")).await?;  // 11：批次 → 各工步过站数
match s.serial_no_info(front, "CIS-CH11HG260904940XAG0010", None, None).await? {       // 12：条码详细信息
  Some(info) => println!("条码 {} 批次 {}，容量 {}", info.sn, info.moLotNo, info.snQty),
  // 条码不存在时服务端 result 为 null 且 success 仍为 true —— 这是「查不到」，不是异常
  None => println!("条码不存在"),
}
```

**测试（真实输出）**：

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

## 📊 接口总览（22 个 CLI task）

### CMES（17 个）

| 模块 | `--task` | 说明 |
|------|----------|------|
| `cmes::login` | `User:CreateToken` | 登录（账号带 `@` 走 MD5，否则 Base64） |
| `cmes::test_scan` | `TestScan:CheckLotSnCode` | SN 校验过站 |
| `cmes::test_scan` | `TestScan:GetInfoBySn` | 根据 SN 获取批次信息 |
| `cmes::test_scan` | `TestScan:GetProduceDetails` | 查询测试项结果（**GET**，411 修复） |
| `cmes::test_scan` | `TestScan:SaveProduceTestByAutoTest` | 保存测试结果（报工） |
| `cmes::record` | `Record:InsertAtSpi` | 序列扫描过站 |
| `cmes::feed_sheet` | `FeedSheet:GetFeedSheetList` | 查询投料单（moLotNo=批次号/SN） |
| `cmes::pack` | `Pack:GetOtherListBySn` | **按产品条码获取组件关联列表**（对接标准 2026-09） |
| `cmes::pack` | `Pack:GetSnPackFinishQty` | **按批次获取 SN 包装完工数**（整机/部件分行） |
| `cmes::bas` | `FeedSheetOrder:GetFeedSheetByContainerName` | **V1.1 第 10 章**：扫描条码 → 所属批次投料单物料行（应发数 > 0，按物料编码倒序） |
| `cmes::bas` | `ScanHistory:GetStationPassQty` | **V1.1 第 11 章**：批次 → 各工步过站数（整机/部件分行，按条码去重；`moLotNo` ≤ 100 位、`stationCodes` ≤ 50 个前置校验）；不传 `stationCodes` 返回该批次**全部工步** |
| `cmes::bas` | `ScanHistory:VerifyStationPassQty` | **过站数闭环验收**：先查全部工步 → 再指定工站复查 → 逐项对比（只返回指定工站 / 过站数与全量一致 / 行数一致 / 抽查时须收窄），并校验全量结果自身口径（`moLotNo` 与请求一致、`moQty` 一致、`sortNo` 升序、同工步整机在部件前）；`verdict` 非 PASS 时进程失败，验证表格显示 FAIL |
| `cmes::bas` | `SerialNoCache:GetInfoBySn` | **V1.1 第 12 章**：条码详细信息；**条码不存在时 `result` 为 `null` 且不算异常**（库返回 `Option::None`） |
| `cmes` | `CMES:MoLotNo` | 高层封装：SN → 批次号（登录+校验链式） |
| `cmes` | `CMES:SessionToken` | 返回 Session 缓存的 Token（验证会话层机制） |
| `cmes` | `CMES:Report` | 高层封装：SN → 报工（登录+校验+报工链式） |
| `cmes::session` | `SelfTest:Session` | **内置自检**：本地 mock 验证 Token 缓存/重试/分层解析/blocking，**不访问生产** |
| `cmes::login` | `Diagnose:Auth <url>` | **前端 API 鉴权诊断**：同一 Token 依次用 5 种头访问，定位服务端认哪种鉴权（Token 不落日志） |

### SCM WIP（2 个，MD5 网关签名）

| 模块 | `--task` | 说明 |
|------|----------|------|
| `scm::wip` | `SCM:MoFactory` | 查询工单信息（**对齐官方 DefaultSignSDK**：仅 String 参与签名 + 结构体 null 容错 + 错误附原始响应） |
| `scm::wip` | `SCM:ReqCommon` | 查询投料单信息（同上签名约定） |

### ItAPI（3 个）

| 模块 | `--task` | 说明 |
|------|----------|------|
| `itapi` | `ItAPI:BatchProdDocs` | 批量查询 CPLM 生产文件 |
| `itapi` | `ItAPI:SoftInfoListV2` | 查询软件信息表 |
| `itapi` / `mac` | `ItAPI:MacQuery` | 按 SN 查询 MAC 绑定 |

### 库函数（任意位置调用）

| 模块 | 函数 | 说明 |
|------|------|------|
| `mac` | `query_macs_by_sn` | SN → MAC 绑定查询（防混料，兼容 CMES / ItAPI 鉴权） |

---

## 🔐 CMES 对接标准（V1.1 / 2026-10-14）

| 环节 | 对接标准 | 库内实现 |
|------|----------|----------|
| 取 Token | `GET {域名}/api/Account/CreateToken?Email=&Pwd=`（Pwd 为 32 位小写 MD5 摘要） | `login::login` |
| 携带 Token | 请求头 `x-auth-token: {Token}`；Header 名不区分大小写 | `session::auth_headers`（`Cfg::tokenName` 非空时以现场配置为准，留空回退标准名） |
| Token 生命周期 | 默认 2 小时；过期前主动刷新；多线程/多实例并发刷新需收敛 | `session::Session`（提前 5 分钟刷新，并发调用只登录一次） |
| Token 失效 | HTTP 401 或 `unAuthorizedRequest === true` → 清除旧 Token、重取、原请求**最多重试一次** | `Session::get_json`；重取仍失败即停止并告警 |
| HTTP 500 | 系统异常或服务端友好提示：读 `error.message` / `error.details` 或响应体文本，**不解析 `result`** | `session::classify`（提示可直接展示给操作员） |
| HTTP 200 + `success !== true` | 业务处理失败：读 `error`，不使用 `result` | 同上 |
| HTTP 200 + `success === true` | 业务成功：业务数据取 `result` 字段 | `session::classify` |
| 其他状态码 / 非 JSON | 未约定的接口异常 / 协议异常（记录状态码与脱敏后的原始响应） | 同上 |
| 参数错误 | `snCode` / `moLotNo` / `containerName` / `sn` 去空白后为空 → 不发起请求；`moLotNo` 超 100 位、`stationCodes` 超 50 个同样前置拒绝 | `pack` / `bas` 各接口前置校验（`check_sn` / `check_lot` / `check_container` / `check_station_codes`） |
| 「查不到」≠ 异常 | `bas/SerialNoCache/GetInfoBySn`：**条码不存在时 `success` 仍为 `true` 且 `result` 为 `null`**，须以 `result` 是否为 `null` 判断，**不得按 `error` 解析** | `bas::serial_no_info` → `Ok(None)`；由 `parse_serial_info` + 端到端单测固定 |
| null 容错 | 第 12 章 `isIncludeOrder=false` 时多个字段以 `null` 返回（文档逐字段标注不完整，实测单据类 `orderId`/`orderNo`/`moLotNo` 亦为 `null`） | `bas::without_nulls` 统一按「`null` ≡ 缺失 ≡ 默认值」处理，避免一个 `null` 让整行解析失败 |
| 限流 | `GetSnPackFinishQty` / `GetFeedSheetByContainerName` / `GetStationPassQty` 各按 `moLotNo`、`containerName`、`moLotNo` 维度限流（默认 10 秒窗口 30 次）；`GetInfoBySn` 无限流 | 调用方控制频率，勿在循环里高频重试 |
| 安全 | 账号、密码、MD5 摘要、Token 禁止写入日志或错误提示；Token 只走 Header 不拼 URL | 库内错误信息不含凭据；调用方按最小权限分配对接账号 |

---

## 🖥 CLI 联调示例

```sh
# CMES 测试登录（真实环境）
cargo run -- --api CMES --task User:CreateToken --setting '{"stationName":"测试","prodUrl":"http://url.cn","sysUrl":"http://url.cn","userName":"uname","passWord":"passwd","tokenName":"cmes_test_info","causeNo":"test","workCenter":"TEST01","testFrameidNo":"TEST","stationCode":"test"}'

# 按产品条码取组件关联列表（--args[0]=前端 API 根，--args[1]=产品条码）
cargo run -- --api CMES --task Pack:GetOtherListBySn --args 'http://heg-pc-cmes.gz.cvte.cn/frontApi/prod' 'U23070429-0A03693' --setting '<CMES setting>'

# 按批次取 SN 包装完工数（--args[0]=前端 API 根，--args[1]=批次号）
cargo run -- --api CMES --task Pack:GetSnPackFinishQty --args 'http://heg-pc-cmes.gz.cvte.cn/frontApi/prod' 'HEG-YKCBU2607034701' --setting '<CMES setting>'

# 【V1.1-10】扫描条码取批次投料单物料行（--args[0]=前端 API 根，--args[1]=扫描条码）
cargo run -- --api CMES --task FeedSheetOrder:GetFeedSheetByContainerName --args 'http://heg-pc-cmes.gz.cvte.cn/frontApi/prod' 'CIS-CH11HG260904940XAG0010' --setting '<CMES setting>'

# 【V1.1-11】按批次取各工步过站数（--args[2]=工步编码，可选，逗号分隔，最多 50 个；省略则统计全部工步）
cargo run -- --api CMES --task ScanHistory:GetStationPassQty --args 'http://heg-pc-cmes.gz.cvte.cn/frontApi/prod' 'HEG-YKCBU2607034701' 'B10,B20' --setting '<CMES setting>'

# 【V1.1-12】按条码取条码详细信息（--args[2]=isIncludeOrder 可选 true/false；--args[3]=componentBasisCode 可选）
#  注意：条码不存在时输出 content 为 null 且 status 仍为 true（不是错误）
cargo run -- --api CMES --task SerialNoCache:GetInfoBySn --args 'http://heg-pc-cmes.gz.cvte.cn/frontApi/prod' 'CIS-CH11HG260904940XAG0010' --setting '<CMES setting>'

# 前端 API 鉴权诊断：用登录拿到的 Token 依次用 5 种头访问同一 URL，定位服务端认哪种
#   可在 --args[1..] 追加候选头名（如主站真实头名 cmes_heg_info）：会同时试「同名请求头」与「同名 Cookie」
#   每次探测还回报 resp_headers（WWW-Authenticate / Location / x-* / Set-Cookie 的 cookie 名），
#   并把返回 200 的候选汇总到 ok_candidates —— 一眼看出前端认哪种凭据
cargo run -- --api CMES --task Diagnose:Auth --args 'http://heg-pc-cmes.gz.cvte.cn/frontApi/prod/api/services/pack/Pack/GetOtherListBySn?snCode=U23070429-0A03693' 'cmes_heg_info' --setting '<CMES setting>'

# 内置自检（本地 mock，零外部依赖）
cargo run --features input-api,blocking -- --api CMES --task SelfTest:Session

# SCM 查询工单信息（MD5 签名仅 String 类型参与）
cargo run -- --api SCM --task SCM:MoFactory --setting '{"gateway":"https://itgw-sit.cvte.com/env-101/scm/scmwip/gw-wip","apikey":"<应用ID>","api_secret":"<密钥>"}' --args '{"lineLotNumber":"HEG-SKZBU23070075"}'

# ItAPI 按 SN 查询绑定 MAC
cargo run -- --api ItAPI --task ItAPI:MacQuery --setting '{"query_url":"https://.../GetMacBySn","station_code":"ZS220","apikey":"<应用ID>"}' --args 'SN12345678'
```

---

## 🧪 批量验证脚本（scripts/）

一次跑完全部 31 个只读用例（主账号 16 + 板卡组 9 + SCM 2 + ItAPI 3；`RUN_WRITE=1` 再追加 3 个写入用例），输出彩色 PASS/FAIL 表格：

| 文件 | 用途 | 凭据 |
| --- | --- | --- |
| `scripts/verify.bat` | 通用模板：调 `verify.prod.bat` + 自动从 `etest.db` 生成（若 `cvte-mes-gencfg.exe` 在场） | 无（外引） |
| `scripts/verify.prod.bat` | **生产配置**：账号 / 域名 / 条码；由 `cvte-mes-gencfg.exe` 生成 | ✅ 含凭据，已 gitignore |
| `scripts/verify.prod.board.bat` | **板卡组配置**：与主账号分离的站点/账号（板卡条码归属另一站点/组织，必须独立账号） | ✅ 含凭据，已 gitignore |
| `scripts/gen-cfg.bat` | 用 `cvte-mes-gencfg.exe` 从 `etest.db` 列出候选 → 选 id → 生成 `verify.prod.bat` | 无 |
| `scripts/_render.ps1` | 彩色表格渲染器（被 bat 调用） | 无 |
| `scripts/cvte-mes-gencfg.exe` | 静态 SQLite 读取器（**bundled**，现场无需 Python） | — |
| `scripts/cvte-mes.exe` | CLI 主程序（拷自 `target\release`） | — |

```bat
:: exe 查找顺序：scripts\cvte-mes.exe → target\release → target\debug
cargo build --release --features input-api,blocking
copy target\release\cvte-mes.exe scripts\
scripts\verify.bat
```

- **覆盖范围**：主账号 16 个（`SelfTest:Session`、`User:CreateToken`、6 个只读查询、`FeedSheetOrder`/`ScanHistory`/`SerialNoCache` 三个 V1.1 新增、`SerialNoCache` 的 `isIncludeOrder=false` 空值形态、`CMES:MoLotNo`、`CMES:SessionToken`、`Diagnose:Auth`）+ 板卡组 9 个（同套用例走第二账号/站点，含 `FeedSheetOrder`/`ScanHistory`/`SerialNoCache`，且 `Diagnose:Auth` 会自动多试候选头名）+ SCM 2 个 + ItAPI 3 个 = **31 个**；其中 `Record:InsertAtSpi` / `TestScan:SaveProduceTestByAutoTest` / `CMES:Report` 三个写入接口默认跳过，`RUN_WRITE=1` 才执行（合计 34 个）
- **用例跳过条件**：主账号组需配置 `SN`（`SN` 为空则整段跳过，`CMES:SessionToken`/`Diagnose:Auth` 仍执行）；板卡组需配置 `SN2`（为空则整段跳过）
- **生产配置自动加载**：`verify.bat` 检测到 `verify.prod.bat` 自动 call；缺失且同目录有 `cvte-mes-gencfg.exe` + `etest.db` 时自动生成；主配置缺项由脚本兜底（如 `PARENT_ID` 默认值）
- **板卡组双账号**：`verify.prod.board.bat` 中任意项未配置 → 自动回退主配置；板卡条码（B26082334-* 这类）必须用板卡专用凭据
- **域名兜底**：`PROD_URL2` / `SYS_URL2` / `FRONT_URL2` 未配置时回退到 `PROD_URL` / `SYS_URL` / `FRONT_URL`
- **输出**：逐项 `PASS`(绿) / `FAIL`(红) / `ERROR`(黄) + 汇总 + 失败详情；每个接口的原始响应留在 `scripts\verify.runs\`；本轮日志按 `logs\<N>\` 归档（result.txt + runs 全量）
- **退出码**：有失败为 `1`，全绿为 `0`；窗口跑完不自动关闭
- **可整包拷贝**：`scripts\` 拷到内网机器直接双击运行（不依赖项目根）

### 内置自检（不依赖生产环境）

脚本构造不出的场景——Token 缓存、并发只登录一次、401 自动重取并**最多重试一次**、HTTP 500 / 业务失败 / 非 JSON 的分层解析、blocking 同步路径，以及新增的 V1.1 第 10~12 章三接口解析（含「条码不存在 → `Ok(None)`」约定）——由程序内置自检覆盖：它在本机起一个 mock 服务端按文档真实报文应答，**不访问生产、不产生业务副作用**。

```bat
cargo run --features input-api,blocking -- --api CMES --task SelfTest:Session
```

覆盖 16 项检查（任意一项失败 `cargo test` 即红）：

| # | 检查 | 对应约定 |
|---|------|----------|
| 1 | 连续取 Token 只登录一次且值相同 | 文档 6 · Token 缓存 |
| 2 | 4 个并发取 Token 仍只登录一次 | 文档 6 · 并发只登录一次 |
| 3 | HTTP 401 → 自动重取并重试原请求一次 → 成功 | 文档 4.3 / 5.2 |
| 4 | 持续 401 时只刷新一次即报错（不无限重试） | 文档 5.2 |
| 5 | 显式 Token 路径把 401 识别为「Token 已失效」 | 文档 4.3 |
| 6 | HTTP 500 读取服务端提示（不解析 result） | 文档 4.3 顺序 1 |
| 7 | HTTP 200 且 success=false → 用 error.message | 文档 4.3 顺序 3 |
| 8 | HTTP 200 但非 JSON → 协议异常 | 文档 4.3 顺序 6 |
| 9 | 正常 HTTP 200 成功取 result | 文档 4.3 顺序 4 |
| 10 | 第 10 章：条码 → 投料单物料行解析正确（含 bool / 小数） | V1.1 文档 10.3 |
| 11 | 第 11 章：各工步过站行解析正确，`stationCode` 为 `null` 亦不误判 | V1.1 文档 11.4 |
| 12 | 第 12 章：条码详细信息对象解析正确 | V1.1 文档 12.3 |
| 13 | 第 12 章：**条码不存在（`result` 为 `null`）返回 `Ok(None)` 而非报错** | V1.1 文档 12.2 |
| 14 | 空白 `containerName` 直接报参数错误，不发起请求 | V1.1 文档 10.4 |
| 15 | 第 11 章**闭环验收**：全量工站 → 指定工站复查，两侧逐项一致且抽查须收窄 | V1.1 文档 11.1 / 11.2 |
| 16 | blocking 同步路径可用（需 blocking feature） | GUI 场景 |

### 生产配置来源（现场无需 Python）

`cvte-mes-gencfg.exe` 从 `etest.db` 的 `org_cvte_cmes` 表读取账号/域名，生成 `verify.prod.bat`（含明文凭据，已 gitignore）；`verify.bat` 检测到它存在会自动加载，换账号/换工厂不必改脚本本体。SQLite 是**静态编译**进该 exe 的，现场无需 Python 或任何外部依赖。

```bat
:: 需要重新生成配置时（开发机构建一次，产物可直接拷到现场）
cargo build --release --features gen-cfg

:: 现场双击 gen-cfg.bat：列出候选 → 选 id → 生成 verify.prod.bat
gen-cfg.bat
gen-cfg.bat D:\路径\etest.db        :: 或指定 etest.db 路径
```

开发机上也保留了 Python 版：

```sh
python scripts/gen_prod_cfg.py --list
python scripts/gen_prod_cfg.py --id 1 --sn <SN> --lot <LOT>
```

---

## 🐧 Linux 构建（Kylin V10 / UOS V20 / Ubuntu 18.04+）

```bash
cargo build --release --features input-api,blocking
```

> `input-api` 是 CLI 所需的 feature（`src/main.rs` 依赖 `cvte_mes::input`）；不带该 feature 构建 bin 会报 `could not find input in cvte_mes`。

- reqwest 0.13 + rustls/ring：纯 Rust TLS，无需系统 OpenSSL / CMake，HTTPS 与交叉编译都不额外依赖
- Windows 上交叉 `check --target x86_64-unknown-linux-gnu` 可直接通过（无 openssl-sys）
- rust-version ≥ 1.85（reqwest 0.13 要求）

---

## 📁 目录结构

```
cvte-mes/
├── src/
│   ├── mes/
│   │   ├── cmes/        # CMES：login / test_scan / record / feed_sheet / pack / bas / session
│   │   ├── scm/         # SCM WIP：wip + sign（MD5 网关签名）
│   │   ├── itapi/       # ItAPI：CPLM 生产文件 / 软件信息表
│   │   └── mac.rs       # SN→MAC 绑定（兼容 CMES / ItAPI 鉴权）
│   ├── input/api.rs     # CLI 任务分发
│   ├── blocking.rs      # 同步封装（feature = "blocking"）
│   ├── selftest.rs      # 内置自检（本地 mock）
│   └── bin/gen_cfg.rs   # cvte-mes-gencfg（feature = "gen-cfg"）
├── scripts/             # 批量验证脚本（见 §"批量验证脚本"）
├── doc/                 # 对接方接口文档归档（CMES / SCM / ItAPI / 官方签名 Demo）
└── Cargo.toml
```

---

## 🔧 Development

- 提交规范：`feat` / `fix` / `chore` / `docs` / `refactor` / `test` 前缀
- 新接口：按现有模块模式（Cfg + 请求 + 解析 + 单测）；CMES 新接口统一走 `session::get_json_with_token`，字段名以接口实际返回为准
- SCM 签名对齐官方 `DefaultSignSDK`：仅 String 类型参与签名；结构体 `null` 字段容错；错误附原始响应
- V1.1 `bas` 接口解析前统一过 `without_nulls`：`#[serde(default)]` 只兜「字段缺失」，不兜「字段为 null」
- 提交前：`cargo test` 全绿（48/48）+ `cargo clippy --all-features --all-targets` 无新增警告

---

## 📋 更新日志

| 版本 | 日期 | 关键改动 |
|---|---|---|
| **v0.2.0-rc.2** | 2026-10-16 | **《CMES 对外接口对接标准 V1.1（2026-10-14）》第 10~12 章 3 个新接口落地**：新增 `cmes::bas` 模块——`GetFeedSheetByContainerName`（扫描条码 → 所属批次投料单物料行，应发数 > 0、按物料编码倒序）、`GetStationPassQty`（批次 → 各工步过站数，整机/部件分行、按条码去重、`stationCode` 可为 `null`）、`GetInfoBySn`（条码详细信息，**条码不存在时 `result` 为 `null` 且非异常，库返回 `Ok(None)`**）；三接口均支持 `Session` 会话版与 blocking `CmesSession` 版；CLI 新增 `FeedSheetOrder:*` / `ScanHistory:*` / `SerialNoCache:*` 三个 task；前置校验 `containerName`/`sn` 空值、`moLotNo` ≤ 100 位、`stationCodes` ≤ 50 个（并做 `B10, B20` → `B10,B20` 归一化）；新增 `without_nulls` 统一「`null` ≡ 缺失 ≡ 默认值」，修 `isIncludeOrder=false` 时单据类字段为 `null` 导致整行解析失败；**内置自检 `SelfTest:Session` 从 10 项扩到 16 项（含第 11 章闭环验收）**，新增 4 项覆盖第 10~12 章报文解析与「条码不存在 → `Ok(None)`」约定、1 项空参数前置拒绝；测试 35 → 48；验证脚本 19 → 31 个只读用例（含闭环验收 `ScanHistory:VerifyStationPassQty`）（+3 写入用例；含 ch12 `isIncludeOrder=false` 空值形态）；`Diagnose:Auth` 支持 `--args[1..]` 追加候选头名并回报响应头（Set-Cookie 仅留 cookie 名）；修正 `verify.prod.board.bat` 模板换行符（LF → CRLF，否则 cmd 解析错乱） |
| v0.2.0-rc.1 | 2026-09-20 | **《CMES 对外接口对接标准》补全**：新增 `pack` 模块 `GetOtherListBySn`（SN → 组件关联列表）/ `GetSnPackFinishQty`（批次 → 包装完工数，整机/部件分行）；新增 `session` 模块——Token 缓存（2 小时、到期前 5 分钟刷新、并发只登录一次）+ HTTP 401 与 `unAuthorizedRequest` 自动重取并**最多重试一次** + HTTP 500/401/200/未约定/非 JSON 分层解析（提示优先 `error.message` → `error.details`）；CLI 新增 `Pack:*` 两个 task；blocking 新增 `CmesSession` 会话；测试 16 → 35；新增 `scripts/` 批量验证脚本（彩色 PASS/FAIL 表格）；新增内置自检 `SelfTest:Session`（本地 mock，10 项检查）；新增前端 API 鉴权诊断 `Diagnose:Auth`（5 种头依次探测）；修 SCM/ItAPI 缺 rustls provider 直接 panic；SCM 签名对齐官方 `DefaultSignSDK`（仅 String 参与 + null 容错 + 原始响应附错）；`GetProduceDetails` 改 GET 修复 411 |
| v0.1.2 | 2026-08-28 | 高层封装 `cmes::mo_lot_no` / `cmes::report`（登录 → 校验 → 报工链式调用）+ blocking 包装 |
| v0.1.1 | 2026-08-28 | reqwest 改 rustls + ring（HTTPS 保留且跨平台免 OpenSSL） |
| v0.1.0 | 2026-08-14 | 首发：CMES（登录/过站/报工/投料单）+ SCM WIP（MD5 签名）+ ItAPI（生产文件/软件信息表）+ MAC 绑定查询 |

---

## 📄 License

[MIT](LICENSE)

<div align="center"><sub>Built with ❤️ by EternalNight · Gitee + GitHub 同步发布</sub></div>