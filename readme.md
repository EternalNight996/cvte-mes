# 
```sh
# 例子
cargo run -- --help
# 打包包含所有 API
cargo build --release --features build-all
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