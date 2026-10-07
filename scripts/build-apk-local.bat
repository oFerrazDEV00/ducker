@echo off
setlocal enabledelayedexpansion

echo ========================================================
echo        Ducker - Compilador Local de APK (Android)
echo ========================================================
echo.

where java >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERRO] Java (JDK 17) nao encontrado no PATH!
    echo Execute scripts\setup-mobile-env.bat para instalar.
    pause
    exit /b 1
)

if "%ANDROID_HOME%"=="" (
    if exist "%LOCALAPPDATA%\Android\Sdk" (
        set "ANDROID_HOME=%LOCALAPPDATA%\Android\Sdk"
        echo [INFO] ANDROID_HOME definido automaticamente para: !ANDROID_HOME!
    ) else (
        echo [AVISO] Variavel ANDROID_HOME nao definida e Android SDK padrao nao encontrado.
    )
)

echo.
echo Iniciando compilacao do APK...
cd /d "%~dp0..\app"
cargo tauri android build --apk

if %errorlevel% equ 0 (
    echo.
    echo ========================================================
    echo SUCESSO! O arquivo APK foi gerado em:
    echo app\src-tauri\gen\android\app\build\outputs\apk
    echo ========================================================
) else (
    echo.
    echo [ERRO] Falha ao compilar APK localmente.
    echo DICA: Voce tambem pode compilar pelo GitHub Actions (veja .github/workflows/build-mobile.yml).
)
pause
