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
REM  B 优化版  ：用配置的 frontTokenName、走分层解析（生产验证脚本用的就是这条）
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

set "OUT=%~dp0scanhistory.result.txt"
echo 批次号=%LOT_ARG%   工站码=%CODES_ARG%   前端=%FRONT_URL% > "%OUT%"

echo.
echo ============================================================
echo  A. 官方原样：x-auth-token + 原始响应（未解析）
echo     可直接与官方 curl 的返回逐字节对比
echo ============================================================
call :run A_官方原样 ScanHistory:GetStationPassQtyRaw

echo.
echo ============================================================
echo  B. 优化版：frontTokenName + 分层解析
echo     生产验证脚本 verify.bat 用的就是这条
echo ============================================================
call :run B_优化版 ScanHistory:GetStationPassQty

echo.
echo ------------------------------------------------------------
echo 结果已保存：%OUT%
echo 判读：
echo    A、B 都是空数组      -^> 服务端该批次无过站数据（换批次或找 MES 方核实口径）
echo    A 有数据、B 为空     -^> 客户端与官方的差异，请把本文件发回排查
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
echo. >> "%OUT%"
echo ==== %TITLE% ==== >> "%OUT%"
type "%TMP%" >> "%OUT%"
goto :eof