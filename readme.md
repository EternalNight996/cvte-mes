# 
```sh
# 例子
cargo run -- --help
# 打包包含所有 API
cargo build --release --features input-api
```
# CMES Example
```sh
# 测试登录
cargo run -- --api CMES --task User:CreateToken --setting '{\"stationName\":\"测试\",\"prodUrl\":\"http://url.cn\",\"sysUrl\":\"http://url.cn\",\"userName\":\"uname\",\"passWord\":\"passwd\",\"tokenName\":\"cmes_test_info\",\"causeNo\":\"test\",\"workCenter\":\"TEST01\",\"testFrameidNo\":\"TEST\",\"stationCode\":\"test\"}'

# 判断测试工步
cargo run -- --api CMES --task TestScan:CheckLotSnCode --args '条码' --setting '{\"stationName\":\"测试\",\"prodUrl\":\"http://url.cn\",\"sysUrl\":\"http://url.cn\",\"userName\":\"uname\",\"passWord\":\"passwd\",\"tokenName\":\"cmes_test_info\",\"causeNo\":\"test\",\"workCenter\":\"TEST01\",\"testFrameidNo\":\"TEST\",\"stationCode\":\"test\"'

# 根据条码获取批次
cargo run -- --api CMES --task TestScan:GetInfoBySn --args '条码' --setting '{\"stationName\":\"测试\",\"prodUrl\":\"http://url.cn\",\"sysUrl\":\"http://url.cn\",\"userName\":\"uname\",\"passWord\":\"passwd\",\"tokenName\":\"cmes_test_info\",\"causeNo\":\"test\",\"workCenter\":\"TEST01\",\"testFrameidNo\":\"TEST\",\"stationCode\":\"test\"}'

# 保存测试结果
cargo run -- --api CMES --task TestScan:SaveProduceTestByAutoTest --args '"{\"lotSnCode\":\"XHA241590001\",\"isPass\":true,\"testTime\":\"2024-10-28 02:19:44\",\"workerNo\":\"梦游\",\"keyInfo\":\"\",\"autoTestVersion\":\"heg-autotest2-v0.1.0\",\"testStartDate\":\"2024-10-28 02:19:44\",\"testEndDate\":\"2024-10-28 02:19:44\",\"testItem\":\"网络MAC\"}"'  --setting '{\"stationName\":\"测试\",\"prodUrl\":\"http://url.cn\",\"sysUrl\":\"http://url.cn\",\"userName\":\"uname\",\"passWord\":\"passwd\",\"tokenName\":\"cmes_test_info\",\"causeNo\":\"test\",\"workCenter\":\"TEST01\",\"testFrameidNo\":\"TEST\",\"stationCode\":\"test\"}'
# 查询测试项结果
cargo run -- --api CMES --task TestScan:GetProduceDetails --args '{\"produceTestId\":\"\",\"maxResultCount\":\"\"}' --setting '{\"stationName\":\"测试\",\"prodUrl\":\"http://url.cn\",\"sysUrl\":\"http://url.cn\",\"userName\":\"uname\",\"passWord\":\"passwd\",\"tokenName\":\"cmes_test_info\",\"causeNo\":\"test\",\"workCenter\":\"TEST01\",\"testFrameidNo\":\"TEST\",\"stationCode\":\"test\"}'

# 序列扫描过站
cargo run -- --api CMES --task Record:InsertAtSpi --args '条码' --setting '{\"stationName\":\"测试\",\"prodUrl\":\"http://url.cn\",\"sysUrl\":\"http://url.cn\",\"userName\":\"uname\",\"passWord\":\"passwd\",\"tokenName\":\"cmes_test_info\",\"causeNo\":\"test\",\"workCenter\":\"TEST01\",\"testFrameidNo\":\"TEST\",\"stationCode\":\"test\"}'

# 测试
cargo run -- --api CMES --task TestScan:SaveProduceTestByAutoTest --args '"{\"lotSnCode\":\"XHA241590001\",\"isPass\":true,\"testTime\":\"2024-10-28 02:19:44\",\"workerNo\":\"梦游\",\"keyInfo\":\"\",\"autoTestVersion\":\"heg-autotest2-v0.1.0\",\"testStartDate\":\"2024-10-28 02:19:44\",\"testEndDate\":\"2024-10-28 02:19:44\",\"testItem\":\"网络MAC\"}"' --setting '{\"stationName\":\"测试\",\"prodUrl\":\"http://heg-pc-cmesprod.gz.cvte.cn\",\"sysUrl\":\"http://heg-pc-cmessys.gz.cvte.cn\",\"userName\":\"f_00000006@heg-pc.com\",\"passWord\":\"123qweASD\",\"tokenName\":\"cmes_heg_info\",\"causeNo\":\"HG\",\"workCenter\":\"PC01\",\"testFrameidNo\":\"MOD02\",\"stationCode\":\"ZS220\"}'
```


# SCM Example
```sh
# 查询SCM工单信息（文档：CVTE_SCM_WIP）
cargo run -- --api SCM --task SCM:MoFactory --setting '{"gateway":"https://itgw-sit.cvte.com/env-101/scm/scmwip/gw-wip","apikey":"<网关颁发的应用ID>","api_secret":"<网关颁发的密钥>"}' --args '{"lineLotNumber":"HEG-SKZBU23070075"}'

# 查询SCM投料单信息
cargo run -- --api SCM --task SCM:ReqCommon --setting '{"gateway":"https://itgw-sit.cvte.com/env-101/scm/scmwip/gw-wip","apikey":"<应用ID>","api_secret":"<密钥>"}' --args '{"lineLotNumberList":["HEG-SKZBU23070075"]}'
```

# ItAPI Example
```sh
# 批量查询 CPLM 生产文件（文档：获取生产文件和软件信息表接口信息）
cargo run -- --api ItAPI --task ItAPI:BatchProdDocs --setting '{"base_url":"https://itapi-portal.cvte.com","apikey":"<可选>"}' --args '[{"prodNumber":"001.001.0034469","productName":"P22MB-BO1053B-M","docType":"ALL"}]'

# 根据生产批次ID/编号查询软件信息表
cargo run -- --api ItAPI --task ItAPI:SoftInfoListV2 --setting '{"base_url":"https://itapi-portal.cvte.com","apikey":"<可选>"}' --args '{"lotNumbers":["38"],"moLineIds":["74"]}'
```

# 接口总览（cvte-mes）

| 系统 | 模块 | 接口（CLI task） | 说明 |
|------|------|------------------|------|
| CMES | `mes::cmes` | `User:CreateToken` | 登录 |
| CMES | `mes::cmes` | `TestScan:CheckLotSnCode` | SN 校验过站 |
| CMES | `mes::cmes` | `TestScan:GetInfoBySn` | 根据 SN 获取批次信息 |
| CMES | `mes::cmes` | `TestScan:SaveProduceTestByAutoTest` | 保存测试结果（报工） |
| CMES | `mes::cmes` | `TestScan:GetProduceDetails` | 查询测试项结果 |
| CMES | `mes::cmes` | `Record:InsertAtSpi` | 序列扫描过站 |
| SCM | `mes::scm::wip` | `SCM:MoFactory` | 查询工单信息 |
| SCM | `mes::scm::wip` | `SCM:ReqCommon` | 查询投料单信息 |
| ItAPI | `mes::itapi` | `ItAPI:BatchProdDocs` | 批量查询 CPLM 生产文件 |
| ItAPI | `mes::itapi` | `ItAPI:SoftInfoListV2` | 查询软件信息表 |
| 通用 | `mes::mac` | 库函数 `query_macs_by_sn` | SN→MAC 绑定查询（防混料，兼容 CMES/ItAPI 鉴权） |

# 同步调用（blocking）

GUI 等不便引入 async 运行时的调用方可启用 `blocking` feature：

```toml
cvte-mes = { git = "https://gitee.com/eternalnight996/cvte-mes.git", features = ["blocking"] }
```

```rust
let macs = cvte_mes::blocking::mac_query_by_sn(&mac_cfg, None, "SN12345678")?;
```
