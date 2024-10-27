```sh
# 例子
cargo run -- --help
# 打包包含所有 API
cargo build --release --features build-all
```

```sh
# 运行测试OS
cargo run --features active -- --api Active  --task check
# 打包包含OS API
cargo build --release --features active
```

```sh
# 运行测试Clean清理
cargo run --features clean -- --api Clean  --task empty_recycle_bin
# 打包包含Clean API
cargo build --release --features clean
```

```sh
# https://learn.microsoft.com/zh-cn/windows-hardware/drivers/devtest/devcon-examples
# 运行测试卸载网卡设备, 并重启生效；
cargo run -- --init --api Drive --full --task remove --args /reboot --filter "Intel(R) Ethernet Controller (3) I225-V #4" -- =net
# 运行测试查找网卡驱动信息， 并返回R<Array[JSON格式]>R
cargo run -- --init --api Drive --full --task findnodes --filter "Intel(R) Ethernet Controller (3) I225-V #4" -- =net
# 打包包含OS API
cargo build --release --features drive
```

```sh
# 打印
cargo run --features fileinfo -- --api fileinfo  --task print --args D:\hg-autotest-history\hg-autotest-v3.0.9\tools\heg-tools\heg-upload-mes.exe
# 返回数据
cargo run --features fileinfo -- --api fileinfo  --task all --args D:\hg-autotest-history\hg-autotest-v3.0.9\tools\heg-tools\heg-upload-mes.exe
# 复制依赖出来
cargo run --features fileinfo -- --api fileinfo  --task copy-lib --args E:\Desktop\hg-autotest-v3.0.7\tools\heg-tools\cvte-mes.exe  E:\Desktop\libs
cargo build --release --features fileinfo
```