@echo off
chcp 65001 >nul
setlocal
title 生成生产配置 verify.prod.bat（从 etest.db）

REM ============================================================
REM  从 etest.db 读取生产 MES 配置，生成 verify.prod.bat
REM  用法：双击本文件；或  gen-cfg.bat D:\路径\etest.db
REM  SQLite 已静态编译进 cvte-mes-gencfg.exe，现场无需 Python/其它依赖
REM ============================================================

set "GEN=%~dp0cvte-mes-gencfg.exe"
set "DB=%~1"
if "%DB%"=="" if exist "%~dp0etest.db" set "DB=%~dp0etest.db"
if "%DB%"=="" if exist "F:\MyApp\eternal\etest\etest.db" set "DB=F:\MyApp\eternal\etest\etest.db"

if not exist "%GEN%" (
  echo [ERROR] 未找到 cvte-mes-gencfg.exe（应与本脚本同目录）
  echo         构建：cargo build --release --features gen-cfg
  pause
  exit /b 1
)
if "%DB%"=="" (
  echo [ERROR] 没找到 etest.db
  echo         用法：gen-cfg.bat [etest.db 路径]
  pause
  exit /b 1
)

echo 使用的 etest.db：%DB%
echo.
echo ==================== 候选配置 ====================
"%GEN%" --db "%DB%" --list
echo.
set /p "PICK=要使用的 id（直接回车 = 自动取最后一条可用配置）："
echo.

if "%PICK%"=="" (
  "%GEN%" --db "%DB%" --out "%~dp0verify.prod.bat"
) else (
  "%GEN%" --db "%DB%" --id %PICK% --out "%~dp0verify.prod.bat"
)

echo.
echo 完成。verify.bat 会自动加载本目录的 verify.prod.bat（第二账号配置见 verify.prod.board.bat）
echo 注意：该文件含明文密码，请勿外发。
pause
