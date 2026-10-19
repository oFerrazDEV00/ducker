@echo off
setlocal enabledelayedexpansion

:: =============================================================================
:: Ducker Installer Build Script (Windows)
:: "Be Simple, Be Duck."
::
:: This script:
:: 1. Builds the CLI binary in release mode
:: 2. Copies the CLI binary to the Tauri resources directory
:: 3. Builds the Tauri desktop app with NSIS installer
::
:: Usage:
::   scripts\build-installer.bat           (builds NSIS installer)
::   scripts\build-installer.bat --debug   (debug build for testing)
:: =============================================================================

echo.
echo  ====================================
echo   Ducker Installer Builder
echo   Be Simple, Be Duck.
echo  ====================================
echo.

set "PROJECT_ROOT=%~dp0.."
set "BUILD_MODE=release"
set "BUILD_FLAG=--release"
set "TAURI_BUILD_FLAG="

:: Parse arguments
if "%1"=="--debug" (
    set "BUILD_MODE=debug"
    set "BUILD_FLAG="
    set "TAURI_BUILD_FLAG=--debug"
    echo [INFO] Building in DEBUG mode
) else (
    echo [INFO] Building in RELEASE mode
)

:: Step 1: Build the CLI binary
echo.
echo [1/3] Building Ducker CLI...
cargo build -p ducker-cli %BUILD_FLAG%
if !ERRORLEVEL! neq 0 (
    echo [ERROR] Failed to build Ducker CLI
    exit /b 1
)
echo [OK] CLI built successfully

:: Step 2: Copy CLI binary to Tauri resources
echo.
echo [2/3] Preparing installer resources...

set "CLI_SOURCE=%PROJECT_ROOT%\target\%BUILD_MODE%\ducker.exe"
set "RESOURCES_DIR=%PROJECT_ROOT%\app\src-tauri\resources"

if not exist "%CLI_SOURCE%" (
    echo [ERROR] CLI binary not found: %CLI_SOURCE%
    exit /b 1
)

copy /Y "%CLI_SOURCE%" "%RESOURCES_DIR%\ducker.exe" >nul
if !ERRORLEVEL! neq 0 (
    echo [ERROR] Failed to copy CLI binary to resources
    exit /b 1
)
echo [OK] CLI binary copied to resources

:: Copy LICENSE to resources for reference
copy /Y "%PROJECT_ROOT%\LICENSE" "%RESOURCES_DIR%\LICENSE" >nul
echo [OK] LICENSE copied to resources

:: Step 3: Build the Tauri app with installer
echo.
echo [3/3] Building Ducker Desktop installer...
echo       (this may take several minutes on first build)
echo.

cd /d "%PROJECT_ROOT%\app"

:: Check if npm dependencies are installed
if not exist "node_modules" (
    echo [INFO] Installing npm dependencies...
    call npm install
)

:: Build with Tauri
call npx tauri build %TAURI_BUILD_FLAG%
if !ERRORLEVEL! neq 0 (
    echo [ERROR] Tauri build failed
    exit /b 1
)

echo.
echo  ====================================
echo   Build Complete!
echo  ====================================
echo.

:: List output artifacts
echo [OUTPUT] Generated artifacts:
echo.

set "NSIS_DIR=%PROJECT_ROOT%\target\%BUILD_MODE%\bundle\nsis"
set "MSI_DIR=%PROJECT_ROOT%\target\%BUILD_MODE%\bundle\msi"

if exist "%NSIS_DIR%" (
    echo  NSIS Installer:
    for %%f in ("%NSIS_DIR%\*.exe") do (
        echo    %%f
        for %%a in ("%%f") do echo    Size: %%~za bytes
    )
)

if exist "%MSI_DIR%" (
    echo  MSI Installer:
    for %%f in ("%MSI_DIR%\*.msi") do (
        echo    %%f
        for %%a in ("%%f") do echo    Size: %%~za bytes
    )
:: Copy to dist folder in project root for convenience
if not exist "%PROJECT_ROOT%\dist" mkdir "%PROJECT_ROOT%\dist"
if exist "%NSIS_DIR%\*.exe" copy /Y "%NSIS_DIR%\*.exe" "%PROJECT_ROOT%\dist\" >nul
if exist "%MSI_DIR%\*.msi" copy /Y "%MSI_DIR%\*.msi" "%PROJECT_ROOT%\dist\" >nul

echo.
echo [COPIED] Installers also available in: %PROJECT_ROOT%\dist\
echo.
echo  Be Simple, Be Duck.

echo.
