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
REM 默认用脚本同目录的 cvte-mes.exe；没有则回退项目 target\release → target\debug
set "PROD_URL=http://xxx-cmesprod.gz.cvte.cn"
set "SYS_URL=http://xxx-cmessys.gz.cvte.cn"
set "USER_NAME=你的账号"
set "PASS_WORD=你的密码"
set "TOKEN_NAME=cmes_test_info"
set "STATION_NAME=测试"
set "CAUSE_NO=test"
set "WORK_CENTER=TEST01"
set "TEST_FRAME_ID=TEST"
set "STATION_CODE=test"

REM ---------------- 配置区 2/3：被测数据 ----------------
set "SN=U23070429-0A03693"
set "LOT=HEG-YKCBU2607034701"
set "FRONT_URL=http://xxx-pc-cmes.gz.cvte.cn/frontApi/prod"

REM ---------------- 配置区 3/3：写入类接口（会真写 MES 数据）1=执行 0=跳过 ----------------
set "RUN_WRITE=0"

REM ---------------- 配置区 4/4：SCM / ItAPI（网关凭据留空则该组用例预期失败） ----------------
set "SCM_GATEWAY=https://itgw-sit.cvte.com/env-101/scm/scmwip/gw-wip"
set "SCM_APIKEY="
set "SCM_SECRET="
set "SCM_LINE_LOT="
set "ITAPI_BASE=https://itapi-portal.cvte.com"
set "ITAPI_APIKEY="
set "ITAPI_PROD_NUMBER="
set "ITAPI_LOT_NUMBERS=[]"
set "ITAPI_MO_LINE_IDS=[]"
set "MAC_QUERY_URL="
set "MAC_STATION_CODE="
set "MAC_TOKEN_NAME="
set "MAC_APIKEY="
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
      echo [ERROR] 未找到 cvte-mes.exe：请把它放到 %~dp0，或执行 cargo build --release --features input-api
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

REM ---------------- 占位符防呆 ----------------
echo %PROD_URL%%SYS_URL%%FRONT_URL% | findstr /c:"xxx" >nul 2>&1
if not errorlevel 1 echo [WARN] 配置区还有 xxx 占位符未填（PROD_URL / SYS_URL / FRONT_URL），相关接口会失败
set "SETTING={\"stationName\":\"%STATION_NAME%\",\"prodUrl\":\"%PROD_URL%\",\"sysUrl\":\"%SYS_URL%\",\"userName\":\"%USER_NAME%\",\"passWord\":\"%PASS_WORD%\",\"tokenName\":\"%TOKEN_NAME%\",\"causeNo\":\"%CAUSE_NO%\",\"workCenter\":\"%WORK_CENTER%\",\"testFrameidNo\":\"%TEST_FRAME_ID%\",\"stationCode\":\"%STATION_CODE%\"}"

REM ---------------- 只读用例 ----------------
set /a IDX+=1
echo %IDX%,User:CreateToken,登录取 Token>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task User:CreateToken --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,TestScan:CheckLotSnCode,条码过站校验>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task TestScan:CheckLotSnCode --args "%SN%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,TestScan:GetInfoBySn,条码查批次>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task TestScan:GetInfoBySn --args "%SN%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,TestScan:GetProduceDetails,查询测试项结果>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task TestScan:GetProduceDetails --args "{\"produceTestId\":\"\",\"maxResultCount\":\"10\"}" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,Pack:GetOtherListBySn,条码查组件关联>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task Pack:GetOtherListBySn --args "%FRONT_URL%" "%SN%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

set /a IDX+=1
echo %IDX%,Pack:GetSnPackFinishQty,批次查包装完工数>"%RUNDIR%\%IDX%.tsv"
"%EXE%" --api CMES --task Pack:GetSnPackFinishQty --args "%FRONT_URL%" "%LOT%" --setting "%SETTING%" > "%RUNDIR%\%IDX%.txt" 2>&1

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
goto :eof
