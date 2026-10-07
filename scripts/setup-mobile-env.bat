@echo off
setlocal enabledelayedexpansion

echo ========================================================
echo        Ducker - Instalador de Dependencias Mobile
echo ========================================================
echo.
echo Este script instala o JDK 17 e os targets Rust para Android.
echo.

echo 1. Instalando Microsoft OpenJDK 17 via Winget...
winget install --id Microsoft.OpenJDK.17 -e --accept-package-agreements --accept-source-agreements

echo.
echo 2. Adicionando targets Android no Rustup...
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

echo.
echo 3. Instalando Tauri CLI no Cargo...
cargo install tauri-cli --version ^2.0.0 --locked

echo.
echo ========================================================
echo Instalacao concluida!
echo Para compilar o APK localmente, instale o Android Studio e NDK.
echo Ou faca um push para o GitHub para compilar o APK e IPA automaticamente via Actions!
echo ========================================================
pause
