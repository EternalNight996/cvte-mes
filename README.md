<div align="center">
  <h1>🧩 cvte-mes</h1>
  <p><strong>源控 MES 系统接口库（CMES / SCM WIP / ItAPI 统一封装）</strong></p>
  <p>登录鉴权 · SN 过站 · 报工回传 · 投料单查询 · 组件关联 · 包装完工数 · MAC 绑定 · 软件信息表</p>
</div>

<div align="center">
  📄 [MIT](LICENSE) · 📦 [crates.io](https://crates.io/crates/cvte-mes) · 📚 [docs.rs](https://docs.rs/cvte-mes) · 🌟 [Gitee](https://gitee.com/eternalnight996/cvte-mes) · 🧪 35 tests
</div>

<div align="center">简体中文 | <a href="README.en.md">English</a></div>

---

## ✨ Features

- 🔑 **统一鉴权**：CMES 登录 Token、SCM MD5 签名、ItAPI apikey，三种体系各归其位
- 🔐 **CMES 对接标准**：Token 缓存（2 小时 / 到期前 5 分钟刷新 / 并发只登录一次）、401 与 `unAuthorizedRequest` 自动重取并最多重试一次、HTTP 500/401/200 分层解析（提示优先 `error.message` → `error.details`）
- 🔄 **全异步 + 可选同步**：async 原生支持；GUI 场景启用 `blocking` feature 免运行时（`CmesSession` 会话复用）
- 📦 **投料单建箱**：`GetFeedSheetList` 真实接口（moLotNo 传批次号或 SN），PackPrint 后端建箱取每箱数量
- 🧩 **包装（Pack）接口**：`GetOtherListBySn`（SN → 组件关联列表）、`GetSnPackFinishQty`（批次 → 包装完工数，整机/部件分行）
- 🖥 **CLI 联调工具**：`cargo run -- --api CMES|SCM|ItAPI --task <Task>` 命令行直接验证每个接口；`scripts\verify.bat` 批量跑全部 13 个接口并输出彩色 PASS/FAIL 表格
- 🐧 **跨平台**：Windows / Linux（Kylin V10 / UOS V20 / Ubuntu 18.04+）

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

**测试（真实输出）**：

```sh
$ cargo test --lib
running 35 tests
test mes::cmes::feed_sheet::tests::item_from_json_picks_known_fields ... ok
test mes::cmes::pack::tests::parse_other_items_reads_documented_fields ... ok
test result: ok. 34 passed; 0 failed
```

---

## 📊 接口总览

| 系统 | 模块 | 接口（CLI task） | 说明 |
|------|------|------------------|------|
| CMES | `mes::cmes` | `User:CreateToken` | 登录（账号带 `@` 走 MD5，否则 Base64） |
| CMES | `mes::cmes` | `TestScan:CheckLotSnCode` | SN 校验过站 |
| CMES | `mes::cmes` | `TestScan:GetInfoBySn` | 根据 SN 获取批次信息 |
| CMES | `mes::cmes` | `TestScan:SaveProduceTestByAutoTest` | 保存测试结果（报工） |
| CMES | `mes::cmes` | `TestScan:GetProduceDetails` | 查询测试项结果 |
| CMES | `mes::cmes` | `Record:InsertAtSpi` | 序列扫描过站 |
| CMES | `mes::cmes` | `ScanHead:GetFeedSheetList` | 查询投料单（moLotNo=批次号/SN） |
| CMES | `mes::cmes::pack` | `Pack:GetOtherListBySn` | **按产品条码获取组件关联列表**（对接标准 2026-09） |
| CMES | `mes::cmes::pack` | `Pack:GetSnPackFinishQty` | **按批次获取 SN 包装完工数**（整机/部件分行） |
| CMES | `mes::cmes::session` | 库函数 `Session` | Token 缓存 + 失效重试 + 响应分层（所有新接口共用） |
| SCM | `mes::scm::wip` | `SCM:MoFactory` | 查询工单信息（MD5 签名） |
| SCM | `mes::scm::wip` | `SCM:ReqCommon` | 查询投料单信息（MD5 签名） |
| ItAPI | `mes::itapi` | `ItAPI:BatchProdDocs` | 批量查询 CPLM 生产文件 |
| ItAPI | `mes::itapi` | `ItAPI:SoftInfoListV2` | 查询软件信息表 |
| 通用 | `mes::mac` | 库函数 `query_macs_by_sn` | SN→MAC 绑定查询（防混料，兼容 CMES/ItAPI 鉴权） |

---

## 🔐 CMES 对接标准（2026-09 整理版）

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
| 参数错误 | `snCode` / `moLotNo` 去空白后为空 → 不发起请求 | `pack::get_other_list_by_sn` / `get_sn_pack_finish_qty` 前置校验 |
| 限流 | `GetSnPackFinishQty` 按 `moLotNo` 限流（默认 10 秒窗口 30 次） | 调用方控制频率，勿在循环里高频重试 |
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

# SCM 查询工单信息
cargo run -- --api SCM --task SCM:MoFactory --setting '{"gateway":"https://itgw-sit.cvte.com/env-101/scm/scmwip/gw-wip","apikey":"<应用ID>","api_secret":"<密钥>"}' --args '{"lineLotNumber":"HEG-SKZBU23070075"}'

# ItAPI 按 SN 查询绑定 MAC
cargo run -- --api ItAPI --task ItAPI:MacQuery --setting '{"query_url":"https://.../GetMacBySn","station_code":"ZS220","apikey":"<应用ID>"}' --args 'SN12345678'
```

---

## 🧪 批量验证脚本（scripts/）

一次跑完全部 13 个接口，输出彩色 PASS/FAIL 表格：

| 文件 | 用途 |
| --- | --- |
| `scripts/verify.bat` | 通用模板：配置区填自己的环境 / 账号 / 条码 |
| `scripts/verify2.bat` | 现场真实账号版（含明文凭据，已在 `.gitignore` 中忽略，**勿提交**） |
| `scripts/_render.ps1` | 彩色表格渲染器，被 bat 调用，无凭据 |

```bat
:: exe 查找顺序：scripts\cvte-mes.exe → target\release → target\debug
cargo build --release --features input-api,blocking
copy target\release\cvte-mes.exe scripts\
scripts\verify.bat
```

- 覆盖范围：CMES 8 个（含 `Record:InsertAtSpi`、`TestScan:SaveProduceTestByAutoTest` 两个写入接口，默认跳过，`RUN_WRITE=1` 才执行）+ SCM 2 个 + ItAPI 3 个
- 输出：逐项 `PASS`(绿) / `FAIL`(红) / `ERROR`(黄) + 汇总 + 失败详情；每个接口的原始响应留在 `scripts\verify.runs\`
- 退出码：有失败为 `1`，全绿为 `0`；窗口跑完不自动关闭
- `scripts\` 可整包拷到内网机器直接双击运行（不依赖项目根）

---

### 内置自检（不依赖生产环境）

脚本构造不出的场景——Token 缓存、并发只登录一次、401 自动重取并**最多重试一次**、HTTP 500 / 业务失败 / 非 JSON 的分层解析、blocking 同步路径——由程序内置自检覆盖：它在本机起一个 mock 服务端，**不访问生产、不产生业务副作用**。

```bat
cargo run --features input-api,blocking -- --api CMES --task SelfTest:Session
```

### 生产配置来源

`scripts/gen_prod_cfg.py` 从 `etest.db` 的 `org_cvte_cmes` 表读取账号/域名，生成 `verify.prod.bat`（含明文凭据，已 gitignore）；`verify.bat` 检测到它存在会自动加载，换账号/换工厂不必改脚本本体：

```sh
python scripts/gen_prod_cfg.py --list                        # 列出候选配置
python scripts/gen_prod_cfg.py --id 1 --sn <SN> --lot <LOT>  # 生成
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

## 🔧 Development

- 提交规范：`feat` / `fix` / `chore` / `docs` / `refactor` / `test` 前缀
- 新接口：按现有模块模式（Cfg + 请求 + 解析 + 单测）；CMES 新接口统一走 `session::get_json_with_token`，字段名以接口实际返回为准
- 提交前：`cargo test` 全绿 + `cargo clippy --all-features --all-targets` 无新增警告

---

## 📋 更新日志

| 版本 | 日期 | 关键改动 |
|---|---|---|
| **v0.2.0-rc.1** | 2026-09-20 | **《CMES 对外接口对接标准》补全**：新增 `pack` 模块 `GetOtherListBySn`（SN → 组件关联列表）/ `GetSnPackFinishQty`（批次 → 包装完工数，整机/部件分行）；新增 `session` 模块——Token 缓存（2 小时、到期前 5 分钟刷新、并发只登录一次）+ HTTP 401 与 `unAuthorizedRequest` 自动重取并**最多重试一次** + HTTP 500/401/200/未约定/非 JSON 分层解析（提示优先 `error.message` → `error.details`）；CLI 新增 `Pack:*` 两个 task；blocking 新增 `CmesSession` 会话；测试 16 → 34；新增 `scripts/` 批量验证脚本（13 个接口彩色 PASS/FAIL 表格）；修 SCM/ItAPI 缺 rustls provider 直接 panic |
| v0.1.2 | 2026-08-28 | 高层封装 `cmes::mo_lot_no` / `cmes::report`（登录 → 校验 → 报工链式调用）+ blocking 包装 |
| v0.1.1 | 2026-08-28 | reqwest 改 rustls + ring（HTTPS 保留且跨平台免 OpenSSL） |
| v0.1.0 | 2026-08-14 | 首发：CMES（登录/过站/报工/投料单）+ SCM WIP（MD5 签名）+ ItAPI（生产文件/软件信息表）+ MAC 绑定查询 |

---

## 📄 License

[MIT](LICENSE)

<div align="center"><sub>Built with ❤️ by EternalNight</sub></div>
