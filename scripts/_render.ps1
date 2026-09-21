<#
  cvte-mes 验证结果渲染器（被 scripts\verify.bat / verify2.bat 调用）
  读取 <RunDir>\N.tsv（序号,接口,说明）与同序号 N.txt（exe 原始输出），
  输出带颜色的 PASS/FAIL 表格，同时把同样的纯文本结果写到 -ResultPath
  （窗口被误关 / 弹窗吃掉 pause 时，结果依然留在文件里可查）。
  无凭据，可提交。
#>
param(
  [Parameter(Mandatory = $true)][string]$RunDir,
  [string]$Title = 'cvte-mes 接口验证',
  [string]$Env = '',
  [string]$Account = '',
  [string]$ResultPath = ''
)

try { [Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false) } catch { }

$script:lines = New-Object System.Collections.Generic.List[string]
$script:buf = ''

function Out-Line {
  param([string]$Text, [string]$Color = 'Gray', [switch]$NoNewline)
  if ($null -eq $Text) { $Text = '' }
  if ($NoNewline) {
    Write-Host $Text -ForegroundColor $Color -NoNewline
    $script:buf += $Text
  } else {
    Write-Host $Text -ForegroundColor $Color
    [void]$script:lines.Add($script:buf + $Text)
    $script:buf = ''
  }
}

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
  $m = [regex]::Match($Raw, '"' + $Key + '"\s*:\s*"((?:[^"\\]|\\.)*)"', 'Singleline')
  if ($m.Success) {
    $v = $m.Groups[1].Value
    $v = $v -replace '\\r\\n', ' ' -replace '\\n', ' ' -replace '\\r', ' ' -replace '\\t', ' '
    $v = $v -replace '\\"', '"' -replace '\\\\', '\'
    return $v
  }
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
    if ($detail -match '^eyJ[A-Za-z0-9_\-\.]{20,}$') {
      $detail = "JWT Token（已脱敏，长度 $($detail.Length)）"
    }
    if (-not $detail) {
      $first = ($raw -split "`r?`n" | Where-Object { $_.Trim() } | Select-Object -First 1)
      if ($first) { $detail = '无 status 字段（进程异常？）：' + $first.Trim() }
    }
    if ($detail.Length -gt 400) { $detail = $detail.Substring(0, 400) + '...' }

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

Out-Line ''
Out-Line $bar 'DarkGray'
Out-Line (' ' + $Title + '    ' + (Get-Date -Format 'yyyy-MM-dd HH:mm:ss')) 'White'
if ($Env) { Out-Line (' 环境  ' + $Env) 'Gray' }
if ($Account) { Out-Line (' 账号  ' + $Account) 'Gray' }
Out-Line $bar 'DarkGray'
Out-Line ('  ' + (Pad-Right '#' $idxW) + ' ' + (Pad-Right '接口' $nameW) + ' ' + (Pad-Right '结果' $resW) + ' 说明') 'DarkCyan'
Out-Line $thin 'DarkGray'

foreach ($r in $rows) {
  $color = switch ($r.Status) { 'PASS' { 'Green' } 'FAIL' { 'Red' } default { 'Yellow' } }
  Out-Line ('  ' + (Pad-Right $r.Idx $idxW) + ' ' + (Pad-Right $r.Name $nameW) + ' ') -NoNewline
  Out-Line (Pad-Right $r.Status $resW) $color -NoNewline
  Out-Line (' ' + $r.Desc)
}

Out-Line $thin 'DarkGray'
Out-Line ('  通过 ') -NoNewline
Out-Line ('{0}' -f $pass) 'Green' -NoNewline
Out-Line ('   失败 ') -NoNewline
Out-Line ('{0}' -f $fail) 'Red' -NoNewline
if ($err -gt 0) {
  Out-Line ('   异常 ') -NoNewline
  Out-Line ('{0}' -f $err) 'Yellow' -NoNewline
}
if ($null -ne $tMin -and $null -ne $tMax) {
  Out-Line ('   耗时 {0:N1}s' -f (($tMax - $tMin).TotalSeconds)) -NoNewline
}
Out-Line ''
Out-Line $bar 'DarkGray'

$withDetail = @($rows | Where-Object { $_.Detail })
if ($withDetail.Count -gt 0) {
  Out-Line ''
  Out-Line ' 结果明细（PASS 也打印）' 'DarkCyan'
  foreach ($r in $withDetail) {
    if ($r.Status -eq 'PASS') {
      Out-Line ('  [PASS] ' + $r.Name) 'Green'
      Out-Line ('         ' + $r.Detail) 'DarkGreen'
    } else {
      Out-Line ('  [' + $r.Status + '] ' + $r.Name) 'Red'
      Out-Line ('         ' + $r.Detail) 'DarkRed'
    }
  }
}

Out-Line ''

if ($ResultPath) {
  try { $script:lines | Set-Content -LiteralPath $ResultPath -Encoding UTF8 } catch { }
}

if ($fail -gt 0 -or $err -gt 0) { exit 1 }
exit 0
