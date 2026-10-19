@echo off
chcp 65001 >nul
setlocal
title cvte-mes 接口验证

REM ============================================================
REM  cvte-mes 接口批量验证脚本（scripts\verify.bat）
REM  用法：把「配置区」按真实环境改好，双击本文件或执行 scripts\verify.bat
REM  结果：彩色表格逐项 PASS/FAIL，原始输出留在 scripts\verify.runs\
REM ============================================================

REM ---------------- 配置区 1/3：接口信息 ----------------
set "EXE=%~dp0cvte-mes.exe"

REM ---------------- 配置区 3/3：写入类接口（会真写 MES 数据）1=执行 0=跳过 ----------------
set "RUN_WRITE=0"

REM ==================== 以下为脚本本体，不用改 ====================
REM 尽量切到项目根（只在回退 target 时用到）；scripts 目录被单独拷走时允许失败
cd /d "%~dp0.." 2>nul

if not exist "%EXE%" (
  if exist "target\release\cvte-mes.exe" (
    set "EXE=target\release\cvte-mes.exe"
  ) else (
    if exist "target\debug\cvte-mes.exe" (
      set "EXE=target\debug\cvte-mes.exe"
    ) else (
      echo [ERROR] 未找到 cvte-mes.exe：请把它放到 %~dp0，或执行 cargo build --release --features input-api,blocking
      pause
      exit /b 1
    )
  )
)

set "RUNDIR=%~dp0verify.runs"
if exist "%RUNDIR%" rmdir /s /q "%RUNDIR%"
mkdir "%RUNDIR%"
set /a IDX=0

echo 正在验证，请稍候（每个接口最长 10 秒超时）...
echo.

REM ---------------- 生产配置：存在则加载；缺失时用同目录 etest.db 自动生成 ----------------
if exist "%~dp0verify.prod.bat" goto :prod_ready
if not exist "%~dp0etest.db" goto :prod_ready
if not exist "%~dp0cvte-mes-gencfg.exe" goto :prod_ready
echo [INFO] 未找到 verify.prod.bat，正在从同目录 etest.db 生成...
"%~dp0cvte-mes-gencfg.exe" --db "%~dp0etest.db" --out "%~dp0verify.prod.bat"

:prod_ready
if exist "%~dp0verify.prod.bat" call "%~dp0verify.prod.bat"

REM ---------------- prod.bat 缺项兜底（不改 prod.bat，只在其未提供时补默认） ----------------
REM PARENT_ID：前端接口（FeedSheet 等）必填的租户/组织标识。
REM 缺了会被服务端拒：ValidationError - The ParentId field is required.
REM 注意 verify.prod.bat 由 cvte-mes-gencfg 重新生成，所以兜底放在本脚本里，避免被覆盖。
if not defined PARENT_ID set "PARENT_ID=3e5b8a05ea8b44efb3405d2cdf94663e"
REM STATION_NAME：SETTING 的 stationName；prod.bat 未提供时用站点码兜底
if not defined STATION_NAME set "STATION_NAME=%STATION_CODE%"

REM ---------------- 占位符防呆 ----------------
echo %PROD_URL%%SYS_URL%%FRONT_URL% | findstr /c:"xxx" >nul 2>&1
if not errorlevel 1 echo [WARN] 配置区还有 xxx 占位符未填（PROD_URL / SYS_URL / FRONT_URL），相关接口会失败
set "SETTING={\"stationName\":\"%STATION_NAME%\",\"prodUrl\":\"%PROD_URL%\",\"sysUrl\":\"%SYS_URL%\",\"userName\":\"%USER_NAME%\",\"passWord\":\"%PASS_WORD%\",\"tokenName\":\"%TOKEN_NAME%\",\"causeNo\":\"%CAUSE_NO%\",\"workCenter\":\"%WORK_CENTER%\",\"testFrameidNo\":\"%TEST_FRAME_ID%\",\"stationCode\":\"%STATION_CODE%\"}"

REM ============================================================
REM  板卡组专用账号（与主账号分离）
REM  板卡条码（B26082334-* 这类）属于另一站点/组织，用主账号查会「条码不存在」，
REM  必须用板卡专用凭据。配置写在 verify.prod.board.bat（不会被 gencfg 覆盖）。
REM  任意项未配置 -> 回退主配置，保证不填也能跑（只是板卡用例预期查不到）。
REM ============================================================
if exist "%~dp0verify.prod.board.bat" call "%~dp0verify.prod.board.bat"
if not defined USER_NAME2 set "USER_NAME2=%USER_NAME%"
if not defined PASS_WORD2 set "PASS_WORD2=%PASS_WORD%"
if not defined TOKEN_NAME2 set "TOKEN_NAME2=%TOKEN_NAME%"
if not defined CAUSE_NO2 set "CAUSE_NO2=%CAUSE_NO%"
if not defined WORK_CENTER2 set "WORK_CENTER2=%WORK_CENTER%"
if not defined TEST_FRAME_ID2 set "TEST_FRAME_ID2=%TEST_FRAME_ID%"
if not defined STATION_CODE2 set "STATION_CODE2=%STATION_CODE%"
if not defined STATION_NAME2 set "STATION_NAME2=%STATION_NAME%"
if not defined PROD_URL2 set "PROD_URL2=%PROD_URL%"
if not defined SYS_URL2 set "SYS_URL2=%SYS_URL%"
if not defined FRONT_URL2 set "FRONT_URL2=%FRONT_URL%"
if not defined PARENT_ID2 set "PARENT_ID2=%PARENT_ID%"
set "SETTING2={\"stationName\":\"%STATION_NAME2%\",\"prodUrl\":\"%PROD_URL2%\",\"sysUrl\":\"%SYS_URL2%\",\"userName\":\"%USER_NAME2%\",\"passWord\":\"%PASS_WORD2%\",\"tokenName\":\"%TOKEN_NAME2%\",\"causeNo\":\"%CAUSE_NO2%\",\"workCenter\":\"%WORK_CENTER2%\",\"testFrameidNo\":\"%TEST_FRAME_ID2%\",\"stationCode\":\"%STATION_CODE2%\"}"

set /a IDX+=1
echo %IDX%,SelfTest:Session,内置自检（会话机制/响应分层/blocking）>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task SelfTest:Session --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

REM ---------------- 只读用例 ----------------
set /a IDX+=1
echo %IDX%,User:CreateToken,登录取 Token>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task User:CreateToken --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

if "%SN%"=="" goto :skip_sn1

set /a IDX+=1
echo %IDX%,TestScan:CheckLotSnCode,条码过站校验>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task TestScan:CheckLotSnCode --args "%SN%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,TestScan:GetInfoBySn,条码查批次>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task TestScan:GetInfoBySn --args "%SN%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,TestScan:GetProduceDetails,查询测试项结果>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task TestScan:GetProduceDetails --args "{\"produceTestId\":\"%SN%\",\"maxResultCount\":\"10\"}" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,Pack:GetOtherListBySn,条码查组件关联>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task Pack:GetOtherListBySn --args "%FRONT_URL%" "%SN%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,Pack:GetSnPackFinishQty,批次查包装完工数>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task Pack:GetSnPackFinishQty --args "%FRONT_URL%" "%LOT%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,FeedSheet:GetFeedSheetList,CMES 查投料单>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task FeedSheet:GetFeedSheetList --args "%FRONT_URL%" "%PARENT_ID%" "%LOT%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

REM ---------------- V1.1 新增：bas 基础数据接口（第 10~12 章） ----------------
set /a IDX+=1
echo %IDX%,FeedSheetOrder:GetFeedSheetByContainerName,条码查投料单物料（V1.1-10 空数组=该批次无应发投料）>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task FeedSheetOrder:GetFeedSheetByContainerName --args "%FRONT_URL%" "%SN%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,ScanHistory:GetStationPassQty,批次查各工步过站数（V1.1-11 空数组=该批次无过站记录）>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task ScanHistory:GetStationPassQty --args "%FRONT_URL%" "%LOT%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,SerialNoCache:GetInfoBySn,条码详细信息（V1.1-12 content=null 即条码不存在 属正常）>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task SerialNoCache:GetInfoBySn --args "%FRONT_URL%" "%SN%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,SerialNoCache:GetInfoBySn(isIncludeOrder=false),条码详细信息（不带单据 单据类字段为空即正常）>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task SerialNoCache:GetInfoBySn --args "%FRONT_URL%" "%SN%" "false" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,CMES:MoLotNo,CMES 组合 查批次号>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task CMES:MoLotNo --args "%SN%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

:skip_sn1

set /a IDX+=1
echo %IDX%,CMES:SessionToken,CMES 会话 Token 缓存>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task CMES:SessionToken --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

REM ---------------- 板卡组（SN2 留空则整段跳过） ----------------
if "%SN2%"=="" goto :skip_sn2

set /a IDX+=1
echo %IDX%,TestScan:CheckLotSnCode(板卡),板卡过站校验>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task TestScan:CheckLotSnCode --args "%SN2%" --setting "%SETTING2%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,TestScan:GetInfoBySn(板卡),板卡查批次>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task TestScan:GetInfoBySn --args "%SN2%" --setting "%SETTING2%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,CMES:MoLotNo(板卡),板卡组合查批次号>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task CMES:MoLotNo --args "%SN2%" --setting "%SETTING2%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,Pack:GetOtherListBySn(板卡),板卡查组件关联>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task Pack:GetOtherListBySn --args "%FRONT_URL2%" "%SN2%" --setting "%SETTING2%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1 
echo %IDX%,FeedSheet:GetFeedSheetList(板卡),板卡批次查投料单>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task FeedSheet:GetFeedSheetList --args "%FRONT_URL2%" "%PARENT_ID2%" "%LOT2%" --setting "%SETTING2%" > "%RUNDIR%\%IDX%.txt" 2>&1

REM ---------------- V1.1 新增：bas 接口（板卡组） ----------------
set /a IDX+=1
echo %IDX%,FeedSheetOrder:GetFeedSheetByContainerName(板卡),板卡条码查投料单物料（空数组正常）>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task FeedSheetOrder:GetFeedSheetByContainerName --args "%FRONT_URL2%" "%SN2%" --setting "%SETTING2%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,ScanHistory:GetStationPassQty(板卡),板卡批次查各工步过站数（空数组正常）>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task ScanHistory:GetStationPassQty --args "%FRONT_URL2%" "%LOT2%" --setting "%SETTING2%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,SerialNoCache:GetInfoBySn(板卡),板卡条码详细信息（null 即条码不存在 属正常）>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task SerialNoCache:GetInfoBySn --args "%FRONT_URL2%" "%SN2%" --setting "%SETTING2%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,Diagnose:Auth(板卡),诊断 板卡前端API 鉴权头（自动多试头名）>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task Diagnose:Auth --args "%FRONT_URL2%/api/services/pack/Pack/GetOtherListBySn?snCode=%SN2%" "cmes_heg_info" "cmes_hegmb_info" "cmes_heg_mb_info" --setting "%SETTING2%" > "%RUNDIR%\%IDX%.txt" 2>&1
:skip_sn2

set /a IDX+=1
echo %IDX%,Diagnose:Auth,诊断 前端API 鉴权头>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task Diagnose:Auth --args "%FRONT_URL%/api/services/pack/Pack/GetOtherListBySn?snCode=%SN%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

REM ---------------- SCM 用例 ----------------
set "SCM_SETTING={\"gateway\":\"%SCM_GATEWAY%\",\"apikey\":\"%SCM_APIKEY%\",\"api_secret\":\"%SCM_SECRET%\"}"
set /a IDX+=1
echo %IDX%,SCM:MoFactory,SCM 查工单信息>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api SCM --task SCM:MoFactory --args "{\"lineLotNumber\":\"%SCM_LINE_LOT%\"}" --setting "%SCM_SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,SCM:ReqCommon,SCM 查投料单>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api SCM --task SCM:ReqCommon --args "{\"lineLotNumberList\":[\"%SCM_LINE_LOT%\"],\"lotNumberList\":[\"%LOT%\"]}" --setting "%SCM_SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

REM ---------------- ItAPI 用例 ----------------
set "ITAPI_SETTING={\"base_url\":\"%ITAPI_BASE%\",\"apikey\":\"%ITAPI_APIKEY%\"}"
REM MacQuery 的 --setting 会先被解析为 itapi::Cfg，故必须同时带 base_url
set "MAC_SETTING={\"base_url\":\"%ITAPI_BASE%\",\"apikey\":\"%MAC_APIKEY%\",\"query_url\":\"%MAC_QUERY_URL%\",\"station_code\":\"%MAC_STATION_CODE%\",\"token_name\":\"%MAC_TOKEN_NAME%\"}"
set /a IDX+=1
echo %IDX%,ItAPI:BatchProdDocs,ItAPI 批量生产文件>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api ItAPI --task ItAPI:BatchProdDocs --args "[{\"docSubClassList\":[],\"prodNumber\":\"%ITAPI_PROD_NUMBER%\"}]" --setting "%ITAPI_SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,ItAPI:SoftInfoListV2,ItAPI 软件信息表>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api ItAPI --task ItAPI:SoftInfoListV2 --args "{\"lotNumbers\":%ITAPI_LOT_NUMBERS%,\"moLineIds\":%ITAPI_MO_LINE_IDS%}" --setting "%ITAPI_SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,ItAPI:MacQuery,ItAPI SN查MAC>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api ItAPI --task ItAPI:MacQuery --args "%SN%" --setting "%MAC_SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

REM ---------------- 写入用例（默认跳过） ----------------
if "%RUN_WRITE%"=="1" call :write_cases

REM ---------------- 渲染彩色表格（结果同时落盘，防窗口误关后看不到） ----------------
set "RESULT=%~dp0verify.result.txt"
set "RC=0"
where powershell >nul 2>&1
if errorlevel 1 goto :plain

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0_render.ps1" -RunDir "%RUNDIR%" -Title "cvte-mes 接口验证（通用模板）" -Env "%PROD_URL% / %SYS_URL%" -Account "%USER_NAME%" -ResultPath "%RESULT%"
set "RC=%ERRORLEVEL%"
goto :reported

:plain
echo [WARN] 未找到 PowerShell，改用纯文本结果。
call :plain_report > "%RESULT%" 2>&1
type "%RESULT%"

:reported
REM ---------------- 归档本次日志（保留历史，便于事后追溯） ----------------
set /a LN=1
:nextlog
if not exist "%~dp0logs\%LN%" goto :gotlog
set /a LN+=1
goto :nextlog

:gotlog
mkdir "%~dp0logs\%LN%\runs" 2>nul
copy /y "%RESULT%" "%~dp0logs\%LN%\result.txt" >nul 2>&1
xcopy /y /q "%RUNDIR%\*.*" "%~dp0logs\%LN%\runs\" >nul 2>&1
echo 本次日志已归档：logs\%LN%\（result.txt + runs 全量原始输出）

echo 使用 exe：%EXE%
echo 结果文件：%RESULT%
echo 原始输出：%RUNDIR%
echo.
echo 按任意键关闭窗口（结果已写入上面的 txt，窗口被误关也能查）...
pause
exit /b %RC%

REM ---------------- PowerShell 不可用时的纯文本兜底 ----------------
:plain_report
setlocal enabledelayedexpansion
echo ============ 结果（纯文本兜底）============
set "PF=0"
set "FF=0"
for /l %%i in (1,1,%IDX%) do (
  set "META="
  set /p META=<"%RUNDIR%\%%i.tsv"
  findstr /r /c:"status.: true" "%RUNDIR%\%%i.txt" >nul 2>&1
  if errorlevel 1 (
    set /a FF+=1
    echo [FAIL] !META!
  ) else (
    set /a PF+=1
    echo [PASS] !META!
  )
)
echo 通过 !PF!   失败 !FF!
set "FRC=0"
if !FF! GTR 0 set "FRC=1"
endlocal & set "RC=%FRC%"
goto :eof

REM ---------------- PowerShell 不可用时的纯文本兜底 ----------------
:plain_report
setlocal enabledelayedexpansion
echo ============ 结果（纯文本兜底）============
set "PF=0"
set "FF=0"
for /l %%i in (1,1,%IDX%) do (
  set "META="
  set /p META=<"%RUNDIR%\%%i.tsv"
  findstr /r /c:"status.: true" "%RUNDIR%\%%i.txt" >nul 2>&1
  if errorlevel 1 (
    set /a FF+=1
    echo [FAIL] !META!
  ) else (
    set /a PF+=1
    echo [PASS] !META!
  )
)
echo 通过 !PF!   失败 !FF!
set "FRC=0"
if !FF! GTR 0 set "FRC=1"
endlocal & set "RC=%FRC%"
goto :eof

REM ---------------- 写入用例（真写 MES 数据，RUN_WRITE=1 才执行） ----------------
:write_cases
for /f %%i in ('powershell -NoProfile -Command "Get-Date -Format 'yyyy-MM-dd HH:mm:ss'"') do set "NOW=%%i"

set /a IDX+=1
echo %IDX%,Record:InsertAtSpi,序列扫描过站（写入）>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task Record:InsertAtSpi --args "%SN%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,TestScan:SaveProduceTestByAutoTest,报工保存（写入）>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task TestScan:SaveProduceTestByAutoTest --args "{\"lotSnCode\":\"%SN%\",\"isPass\":true,\"testTime\":\"%NOW%\",\"workerNo\":\"verify\",\"keyInfo\":\"\",\"autoTestVersion\":\"verify-bat\",\"testStartDate\":\"%NOW%\",\"testEndDate\":\"%NOW%\",\"testItem\":\"网络MAC\"}" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1
set /a IDX+=1
echo %IDX%,CMES:Report,CMES 组合 报工（写入）>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task CMES:Report --args "{\"lotSnCode\":\"%SN%\",\"isPass\":true,\"testTime\":\"%NOW%\",\"workerNo\":\"verify\",\"keyInfo\":\"\",\"autoTestVersion\":\"verify-bat\",\"testStartDate\":\"%NOW%\",\"testEndDate\":\"%NOW%\",\"testItem\":\"网络MAC\"}" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

goto :eof
