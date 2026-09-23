@echo off
REM ============================================================
REM  板卡组专用配置（与 verify.prod.bat 的主账号彻底分离）
REM
REM  为什么必须分开：
REM    板卡条码（如 B26082334-0A00001）属于**另一站点/组织**，
REM    用主账号（PC-BP 站点）去查只会得到「条码不存在」；
REM    必须用**板卡专用账号**才查得到。
REM
REM  本文件**不会被 cvte-mes-gencfg 重新生成**，可放心手填。
REM  任意项留空 -> verify.bat 自动回退到主配置（verify.prod.bat）的同名项。
REM  本文件含明文凭据，勿提交/外发。
REM ============================================================

REM ---------------- 板卡账号 ----------------
set "USER_NAME2="
set "PASS_WORD2="

REM ---------------- 板卡站点/工位（与主机型不同则填） ----------------
set "TOKEN_NAME2="
set "CAUSE_NO2="
set "WORK_CENTER2="
set "TEST_FRAME_ID2="
set "STATION_CODE2="
set "STATION_NAME2="

REM ---------------- 板卡租户/组织标识 ----------------
REM FeedSheet 等前端接口必填；板卡与主机型可能不同，缺了会报
REM ValidationError - The ParentId field is required.
set "PARENT_ID2="
