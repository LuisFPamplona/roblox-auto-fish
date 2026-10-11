@echo off
setlocal
cd /d "%~dp0"
where node >nul 2>&1
if errorlevel 1 (
    echo Install Node.js LTS from https://nodejs.org
    exit /b 1
)
where cargo >nul 2>&1
if errorlevel 1 (
    echo Install Rust stable from https://rustup.rs
    exit /b 1
)
where npm >nul 2>&1
if errorlevel 1 exit /b 1
where cl >nul 2>&1
if errorlevel 1 (
    echo Open a Visual Studio Developer PowerShell with MSVC and Windows SDK.
    exit /b 1
)
call npm ci --ignore-scripts
if errorlevel 1 exit /b 1
call npm audit --audit-level=high
if errorlevel 1 exit /b 1
call npm run build
if errorlevel 1 exit /b 1
call npm run test:security
if errorlevel 1 exit /b 1
cargo check --manifest-path src-tauri\Cargo.toml --locked
if errorlevel 1 exit /b 1
cargo test --manifest-path src-tauri\Cargo.toml --locked
if errorlevel 1 exit /b 1
call npm run app:build -- -- --locked
if errorlevel 1 exit /b 1
echo Installer: src-tauri\target\release\bundle\nsis
exit /b 0
