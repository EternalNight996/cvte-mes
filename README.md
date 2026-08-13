<div align="center">
  <h1>🧩 cvte-mes</h1>
  <p><strong>源控 MES 系统接口库（CMES / SCM WIP / ItAPI 统一封装）</strong></p>
  <p>登录鉴权 · SN 过站 · 报工回传 · 投料单查询 · MAC 绑定 · 软件信息表</p>
</div>

<div align="center">
  📄 [MIT](LICENSE) · 📦 [crates.io](https://crates.io/crates/cvte-mes) · 📚 [docs.rs](https://docs.rs/cvte-mes) · 🌟 [Gitee](https://gitee.com/eternalnight996/cvte-mes) · 🧪 16 tests
</div>

<div align="center">简体中文 | <a href="README.en.md">English</a></div>

## ✨ Features

- 🔑 **统一鉴权**：CMES 登录 Token、SCM MD5 签名、ItAPI apikey，三种体系各归其位
- 🔄 **全异步 + 可选同步**：async 原生支持；GUI 场景启用 `blocking` feature 免运行时
- 📦 **投料单建箱**：`GetFeedSheetList` 真实接口（moLotNo 传批次号或 SN），PackPrint 后端建箱取每箱数量
- 🖥 **CLI 联调工具**：`cargo run -- --api CMES|SCM|ItAPI --task <Task>` 命令行直接验证每个接口
- 🐧 **跨平台**：Windows / Linux（Kylin V10 / UOS V20 / Ubuntu 18.04+），native-tls 走系统 OpenSSL

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

**测试（真实输出）**：

```sh
$ cargo test --lib
test mes::cmes::feed_sheet::tests::item_from_json_picks_known_fields ... ok
test mes::mac::tests::norm_mac_ignores_separators_and_case ... ok
test result: ok. 16 passed; 0 failed
```

## 📊 接口总览

| 系统 | 模块 | 接口（CLI task） | 说明 |
|------|------|------------------|------|
| CMES | `mes::cmes` | `User:CreateToken` | 登录 |
| CMES | `mes::cmes` | `TestScan:CheckLotSnCode` | SN 校验过站 |
| CMES | `mes::cmes` | `TestScan:GetInfoBySn` | 根据 SN 获取批次信息 |
| CMES | `mes::cmes` | `TestScan:SaveProduceTestByAutoTest` | 保存测试结果（报工） |
| CMES | `mes::cmes` | `TestScan:GetProduceDetails` | 查询测试项结果 |
| CMES | `mes::cmes` | `Record:InsertAtSpi` | 序列扫描过站 |
| CMES | `mes::cmes` | `ScanHead:GetFeedSheetList` | **查询投料单**（2026-08-14 真实接口，moLotNo=批次号/SN） |
| SCM | `mes::scm::wip` | `SCM:MoFactory` | 查询工单信息（MD5 签名） |
| SCM | `mes::scm::wip` | `SCM:ReqCommon` | 查询投料单信息（MD5 签名） |
| ItAPI | `mes::itapi` | `ItAPI:BatchProdDocs` | 批量查询 CPLM 生产文件 |
| ItAPI | `mes::itapi` | `ItAPI:SoftInfoListV2` | 查询软件信息表 |
| 通用 | `mes::mac` | 库函数 `query_macs_by_sn` | SN→MAC 绑定查询（防混料，兼容 CMES/ItAPI 鉴权） |

## 🖥 CLI 联调示例

```sh
# CMES 测试登录（真实环境）
cargo run -- --api CMES --task User:CreateToken --setting '{"stationName":"测试","prodUrl":"http://url.cn","sysUrl":"http://url.cn","userName":"uname","passWord":"passwd","tokenName":"cmes_test_info","causeNo":"test","workCenter":"TEST01","testFrameidNo":"TEST","stationCode":"test"}'

# SCM 查询工单信息
cargo run -- --api SCM --task SCM:MoFactory --setting '{"gateway":"https://itgw-sit.cvte.com/env-101/scm/scmwip/gw-wip","apikey":"<应用ID>","api_secret":"<密钥>"}' --args '{"lineLotNumber":"HEG-SKZBU23070075"}'

# ItAPI 按 SN 查询绑定 MAC
cargo run -- --api ItAPI --task ItAPI:MacQuery --setting '{"query_url":"https://.../GetMacBySn","station_code":"ZS220","apikey":"<应用ID>"}' --args 'SN12345678'
```

## 🐧 Linux 构建（Kylin V10 / UOS V20 / Ubuntu 18.04+）

```bash
sudo apt install -y libssl-dev pkg-config   # native-tls 走系统 OpenSSL
cargo build --release
```

- reqwest 0.13 + native-tls：HTTPS 走系统 openssl（Ubuntu18.04 自带 1.1），避开 rustls-aws-lc 的 CMake≥3.12 要求
- Windows 上交叉 `check --target x86_64-unknown-linux-gnu` 会停在 openssl-sys（找不到 Linux 系统库），属正常；请在 Linux 机器上完成最终构建
- rust-version ≥ 1.85（reqwest 0.13 要求）

## 🔧 Development

- 提交规范：`feat` / `fix` / `chore` / `docs` / `refactor` / `test` 前缀
- 新接口：按现有模块模式（Cfg + 请求 + 解析 + 单测），字段名以接口实际返回为准（多候选 key 兼容）
- 提交前：`cargo test` 全绿 + `cargo clippy` 无新增警告

## 📄 License

[MIT](LICENSE)

<div align="center"><sub>Built with ❤️ by EternalNight</sub></div>
