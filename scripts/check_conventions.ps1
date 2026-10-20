# Automated convention auditor for framework-benchmark (PowerShell).
# Verifies zero emojis, max 200 LOC per file, and marker slop ban.

$ErrorActionPreference = "Stop"
$failed = $false

Write-Host "[1/4] Checking for forbidden emojis..."
$emojiRegex = "[\uD83C-\uDBFF\uDC00-\uDFFF\u2600-\u26FF\u2700-\u27BF]"
$files = git ls-files | Where-Object { $_ -notmatch '^\.agents/skills/' -and (Test-Path $_) }

foreach ($file in $files) {
    $content = Get-Content $file -Raw -Encoding utf8
    if ($content -match $emojiRegex) {
        Write-Host "ERROR: Emojis detected in: $file" -ForegroundColor Red
        $failed = $true
    }
}
if (-not $failed) {
    Write-Host "PASS: Zero emojis found." -ForegroundColor Green
}

Write-Host "[2/4] Checking for files exceeding 200 lines..."
foreach ($file in $files) {
    if ($file -match 'skills-lock\.json$' -or $file -match 'Cargo\.lock$') { continue }
    $lines = (Get-Content $file | Measure-Object -Line).Lines
    if ($lines -gt 200) {
        Write-Host "ERROR: $file exceeds 200 lines ($lines lines)" -ForegroundColor Red
        $failed = $true
    }
}
if (-not $failed) {
    Write-Host "PASS: All files under 200 lines limit." -ForegroundColor Green
}

Write-Host "[3/4] Checking for slop markers..."
$sourceFiles = git ls-files '*.rs', '*.sh', '*.ps1', '*.toml', '*.yml', '*.yaml' | Where-Object { $_ -notmatch '^scripts/check_conventions\.' -and $_ -notmatch '^\.github/ISSUE_TEMPLATE/' }
foreach ($file in $sourceFiles) {
    if (-not (Test-Path $file)) { continue }
    $content = Get-Content $file -Raw
    if ($content -match '\b(TODO|FIXME|XXX|HACK)\b') {
        Write-Host "ERROR: Slop markers detected in: $file" -ForegroundColor Red
        $failed = $true
    }
}
if (-not $failed) {
    Write-Host "PASS: Zero slop markers found." -ForegroundColor Green
}

Write-Host "[4/4] Checking snake_case naming for docs and scripts..."
$docFiles = git ls-files 'docs/*', 'scripts/*'
foreach ($file in $docFiles) {
    $name = Split-Path $file -Leaf
    if ($name -cmatch '[A-Z\s]') {
        Write-Host "ERROR: Non-snake_case filename detected: $file" -ForegroundColor Red
        $failed = $true
    }
}
if (-not $failed) {
    Write-Host "PASS: Filenames follow snake_case convention." -ForegroundColor Green
}

if ($failed) {
    Write-Host "Convention audit FAILED." -ForegroundColor Red
    exit 1
}

Write-Host "Convention audit PASSED." -ForegroundColor Green
exit 0
