@echo off
setlocal

:: Se houver ducker.exe (CLI) em segundo plano ocupando a porta, encerra para liberar para a UI
tasklist /FI "IMAGENAME eq ducker.exe" 2>NUL | find /I /N "ducker.exe">NUL
if "%ERRORLEVEL%"=="0" (
    echo Liberando porta para a interface gráfica...
    taskkill /IM ducker.exe /F >nul 2>nul
    timeout /t 1 /nobreak >nul
)

if exist "%~dp0target\debug\ducker-app.exe" (
    start "" "%~dp0target\debug\ducker-app.exe" %*
) else (
    echo Compilando e abrindo o Ducker...
    cargo run -p ducker-app
)
