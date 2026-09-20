<#
  cvte-mes 验证结果渲染器（被 scripts\verify.bat / verify2.bat 调用）
  读取 <RunDir>\N.tsv（序号,接口,说明）与同序号 N.txt（exe 原始输出），
  输出带颜色的 PASS/FAIL 表格。无凭据，可提交。
#>
param(
  [Parameter(Mandatory = $true)][string]$RunDir,
  [string]$Title = 'cvte-mes 接口验证',
  [string]$Env = '',
  [string]$Account = ''
)

try { [Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false) } catch { }

function Get-DisplayWidth {
  param([string]$Text)
  $w = 0
  for ($i = 0; $i -lt $Text.Length; $i++) {
    $c = [int]$Text[$i]
    if (($c -ge 0x1100 -and $c -le 0x115F) -or ($c -ge 0x2E80 -and $c -le 0xA4CF) -or
        ($c -ge 0xAC00 -and $c -le 0xD7A3) -or ($c -ge 0xF900 -and $c -le 0xFAFF) -or
        ($c -ge 0xFE30 -and $c -le 0xFE6F) -or ($c -ge 0xFF00 -and $c -le 0xFF60) -or
        ($c -ge 0xFFE0 -and $c -le 0xFFE6)) { $w += 2 } else { $w += 1 }
  }
  return $w
}

function Pad-Right {
  param([string]$Text, [int]$Width)
  if ($null -eq $Text) { $Text = '' }
  $d = $Width - (Get-DisplayWidth -Text $Text)
  if ($d -lt 1) { return $Text }
  return $Text + (' ' * $d)
}

function Get-Field {
  param([string]$Raw, [string]$Key)
  $m = [regex]::Match($Raw, '"' + $Key + '"\s*:\s*"(.*?)"\s*[,}]', 'Singleline')
  if ($m.Success) { return ($m.Groups[1].Value -replace '\\r', ' ' -replace '\\n', ' ') }
  return ''
}

$idxW = 3
$nameW = 38
$resW = 6
$lineW = 2 + $idxW + 1 + $nameW + 1 + $resW + 1 + 24

$rows = @()
$tMin = $null
$tMax = $null

if (Test-Path -LiteralPath $RunDir) {
  $files = Get-ChildItem -LiteralPath $RunDir -Filter '*.tsv' -File | Sort-Object { [int]($_.BaseName) }
  foreach ($f in $files) {
    $meta = ((Get-Content -LiteralPath $f.FullName -Encoding UTF8 -Raw) -replace "`r?`n", ' ').Trim() -split ','
    $name = if ($meta.Count -gt 1) { $meta[1].Trim() } else { $f.BaseName }
    $desc = if ($meta.Count -gt 2) { $meta[2].Trim() } else { '' }

    $raw = ''
    $txt = Join-Path $RunDir ($f.BaseName + '.txt')
    if (Test-Path -LiteralPath $txt) {
      $fi = Get-Item -LiteralPath $txt
      if ($null -eq $tMin -or $fi.LastWriteTimeUtc -lt $tMin) { $tMin = $fi.LastWriteTimeUtc }
      if ($null -eq $tMax -or $fi.LastWriteTimeUtc -gt $tMax) { $tMax = $fi.LastWriteTimeUtc }
      $raw = Get-Content -LiteralPath $txt -Encoding UTF8 -Raw
    }

    $status = 'ERROR'
    if ($raw -match '"status"\s*:\s*true') { $status = 'PASS' }
    elseif ($raw -match '"status"\s*:\s*false') { $status = 'FAIL' }

    $detail = Get-Field -Raw $raw -Key 'content'
    if (-not $detail) {
      $first = ($raw -split "`r?`n" | Where-Object { $_.Trim() } | Select-Object -First 1)
      if ($first) { $detail = '无 status 字段（进程异常？）：' + $first.Trim() }
    }
    if ($detail.Length -gt 96) { $detail = $detail.Substring(0, 96) + '...' }

    $rows += [pscustomobject]@{
      Idx = [string][int]$f.BaseName
      Name = $name
      Desc = $desc
      Status = $status
      Detail = $detail
    }
  }
}

$pass = @($rows | Where-Object { $_.Status -eq 'PASS' }).Count
$fail = @($rows | Where-Object { $_.Status -eq 'FAIL' }).Count
$err = @($rows | Where-Object { $_.Status -eq 'ERROR' }).Count
$bar = '=' * $lineW
$thin = '-' * $lineW

Write-Host ''
Write-Host $bar -ForegroundColor DarkGray
Write-Host (' ' + $Title + '    ' + (Get-Date -Format 'yyyy-MM-dd HH:mm:ss')) -ForegroundColor White
if ($Env) { Write-Host (' 环境  ' + $Env) -ForegroundColor Gray }
if ($Account) { Write-Host (' 账号  ' + $Account) -ForegroundColor Gray }
Write-Host $bar -ForegroundColor DarkGray
Write-Host ('  ' + (Pad-Right '#' $idxW) + ' ' + (Pad-Right '接口' $nameW) + ' ' + (Pad-Right '结果' $resW) + ' 说明') -ForegroundColor DarkCyan
Write-Host $thin -ForegroundColor DarkGray

foreach ($r in $rows) {
  $color = switch ($r.Status) { 'PASS' { 'Green' } 'FAIL' { 'Red' } default { 'Yellow' } }
  Write-Host ('  ' + (Pad-Right $r.Idx $idxW) + ' ' + (Pad-Right $r.Name $nameW) + ' ') -NoNewline
  Write-Host (Pad-Right $r.Status $resW) -ForegroundColor $color -NoNewline
  Write-Host (' ' + $r.Desc)
}

Write-Host $thin -ForegroundColor DarkGray
Write-Host ('  通过 ') -NoNewline
Write-Host ('{0}' -f $pass) -ForegroundColor Green -NoNewline
Write-Host ('   失败 ') -NoNewline
Write-Host ('{0}' -f $fail) -ForegroundColor Red -NoNewline
if ($err -gt 0) {
  Write-Host ('   异常 ') -NoNewline
  Write-Host ('{0}' -f $err) -ForegroundColor Yellow -NoNewline
}
if ($null -ne $tMin -and $null -ne $tMax) {
  Write-Host ('   耗时 {0:N1}s' -f (($tMax - $tMin).TotalSeconds)) -NoNewline
}
Write-Host ''
Write-Host $bar -ForegroundColor DarkGray

$failed = @($rows | Where-Object { $_.Status -ne 'PASS' -and $_.Detail })
if ($failed.Count -gt 0) {
  Write-Host ''
  Write-Host ' 失败/异常详情' -ForegroundColor Red
  foreach ($r in $failed) {
    Write-Host ('  [' + $r.Status + '] ' + $r.Name) -ForegroundColor Red
    Write-Host ('        ' + $r.Detail) -ForegroundColor DarkRed
  }
}

Write-Host ''
if ($fail -gt 0 -or $err -gt 0) { exit 1 }
exit 0
