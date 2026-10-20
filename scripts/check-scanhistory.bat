@echo off
chcp 65001 >nul
setlocal
title cvte-mes ScanHistory 双轨对照（官方原样 vs 优化版）

REM ============================================================
REM  用途：同一批次、同一入参，分别用「官方原样」与「优化版」各打一次，
REM        直接对比两者结果，判定「空数组」是客户端问题还是服务端数据问题。
REM
REM  用法：
REM    check-scanhistory.bat                      用配置里的批次，查全部工站
REM    check-scanhistory.bat <批次号>             指定批次，查全部工站
REM    check-scanhistory.bat <批次号> B10,B20     指定批次与工站码
REM
REM  A 官方原样：固定 x-auth-token 头、不解析、原样输出原始响应（与官方 curl 对齐）
REM              Token 由本程序内部登录取到（无需手工提供），并同时给出一条可直接执行的 curl
REM  B 优化版  ：用配置的 frontTokenName、走分层解析（生产验证脚本用的就是这条）
REM
REM  结果：根目录 scanhistory.result.txt（便于直接打开）
REM        并归档到 logs\<N>\（与 verify.bat 同一套编号规则，互不覆盖）
REM ============================================================

set "EXE=%~dp0cvte-mes.exe"
if not exist "%EXE%" set "EXE=%~dp0..\target\release\cvte-mes.exe"
if not exist "%EXE%" (
  echo [ERROR] 未找到 cvte-mes.exe，请放到本目录或 ..\target\release
  pause
  exit /b 1
)

if exist "%~dp0verify.prod.bat" call "%~dp0verify.prod.bat"
if not defined FRONT_URL set "FRONT_URL=http://heg-pc-cmes.gz.cvte.cn/frontApi/prod"
if not defined STATION_NAME set "STATION_NAME=%STATION_CODE%"
if not defined PARENT_ID set "PARENT_ID=3e5b8a05ea8b44efb3405d2cdf94663e"
if not defined FRONT_TOKEN_NAME set "FRONT_TOKEN_NAME=%TOKEN_NAME%"

set "LOT_ARG=%~1"
set "CODES=%~2"
if "%LOT_ARG%"=="" set "LOT_ARG=%LOT%"
if "%LOT_ARG%"=="" set /p LOT_ARG=请输入批次号: 
if "%LOT_ARG%"=="" (
  echo [ERROR] 未提供批次号
  pause
  exit /b 1
)
set "CODES_ARG="
if not "%CODES%"=="" set "CODES_ARG=%CODES%"

set "SETTING={\"stationName\":\"%STATION_NAME%\",\"prodUrl\":\"%PROD_URL%\",\"sysUrl\":\"%SYS_URL%\",\"userName\":\"%USER_NAME%\",\"passWord\":\"%PASS_WORD%\",\"tokenName\":\"%TOKEN_NAME%\",\"frontTokenName\":\"%FRONT_TOKEN_NAME%\",\"causeNo\":\"%CAUSE_NO%\",\"workCenter\":\"%WORK_CENTER%\",\"testFrameidNo\":\"%TEST_FRAME_ID%\",\"stationCode\":\"%STATION_CODE%\"}"

REM ---------------- 归档目录：沿用 verify.bat 的编号规则（取第一个不存在的 N） ----------------
set /a LN=1
:nextlog
if not exist "%~dp0logs\%LN%" goto :gotlog
set /a LN+=1
goto :nextlog

:gotlog
set "LOGDIR=%~dp0logs\%LN%"
mkdir "%LOGDIR%" 2>nul

set "OUT=%~dp0scanhistory.result.txt"
echo 批次号=%LOT_ARG%   工站码=%CODES_ARG%   前端=%FRONT_URL% > "%OUT%"
echo 归档目录=%LOGDIR% >> "%OUT%"

echo.
echo ============================================================
echo  A. 官方原样：x-auth-token + 原始响应（未解析）
echo     Token 由脚本内部自动登录取到，无需手工提供；
echo     输出里的 token / curl_command 为明文，可直接复制执行。
echo ============================================================
call :run A_官方原样 ScanHistory:GetStationPassQtyRaw

echo.
echo ============================================================
echo  B. 优化版：frontTokenName + 分层解析
echo     生产验证脚本 verify.bat 用的就是这条
echo ============================================================
call :run B_优化版 ScanHistory:GetStationPassQty

REM ---------------- 归档 ----------------
copy /y "%OUT%" "%LOGDIR%\result.txt" >nul 2>&1

echo.
echo ------------------------------------------------------------
echo 结果已保存：%OUT%
echo 已归档到：  logs\%LN%\  （result.txt + A/B 各自的原始输出）
echo 判读：
echo    A、B 都是空数组      -^> 服务端该批次无过站数据（换批次或找 MES 方核实口径）
echo    A 有数据、B 为空     -^> 客户端与官方的差异，请把 logs\%LN%\ 整个目录发回排查
echo    A、B 都有数据        -^> 正常；空数组是批次本身的问题
echo ------------------------------------------------------------
pause
exit /b 0

:run
set "TITLE=%~1"
set "TASK=%~2"
set "TMP=%TEMP%\scanhist.tmp"
"%EXE%" --api CMES --task %TASK% --args "%FRONT_URL%" "%LOT_ARG%" %CODES_ARG% --setting "%SETTING%" > "%TMP%" 2>&1
type "%TMP%"
copy /y "%TMP%" "%LOGDIR%\%TITLE%.txt" >nul 2>&1
echo. >> "%OUT%"
echo ==== %TITLE% ==== >> "%OUT%"
type "%TMP%" >> "%OUT%"

REM 对官方原样段做自动判定（依据返回里的 result_kind）
if /i "%TASK%"=="ScanHistory:GetStationPassQtyRaw" (
  findstr /c:"array(len=0)" "%TMP%" >nul 2>&1
  if not errorlevel 1 (
    echo [判定] 官方原样：HTTP 成功但 result 为空数组 -^> 服务端该批次无过站数据
  ) else (
    findstr /c:"result_kind" "%TMP%" >nul 2>&1
    if not errorlevel 1 (
      echo [判定] 官方原样：result 非空 -^> 请看上面的 raw_body
    ) else (
      echo [判定] 官方原样：请求本身失败 -^> 看上面的报错
    )
  )
)
goto :eof