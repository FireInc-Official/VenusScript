# VenusScript Installer
$InstallDir = "C:\VenusScript"
$ExeSource = ".\venus_compiler\target\release\venus_compiler.exe"
$ExeTarget = "$InstallDir\vscript.exe"

Write-Host "Installing VenusScript v1.5..." -ForegroundColor Cyan

# 1. Create Directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Write-Host "Created directory $InstallDir" -ForegroundColor Green
}

# 2. Copy Executable
if (Test-Path $ExeSource) {
    Copy-Item -Path $ExeSource -Destination $ExeTarget -Force
    Write-Host "Copied vscript.exe to $InstallDir" -ForegroundColor Green
} else {
    Write-Host "Error: Cannot find compiler executable at $ExeSource. Run 'cargo build --release' first." -ForegroundColor Red
    exit 1
}

# 3. Add to User PATH
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    $NewPath = "$UserPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    Write-Host "Added $InstallDir to User PATH." -ForegroundColor Green
} else {
    Write-Host "$InstallDir is already in User PATH." -ForegroundColor Yellow
}

Write-Host "`nVenusScript installation completed successfully!" -ForegroundColor Green
Write-Host "IMPORTANT: Please restart your terminal or VS Code to be able to use the 'vscript' command." -ForegroundColor Yellow
