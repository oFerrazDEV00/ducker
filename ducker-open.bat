@echo off
setlocal

:: Se houver ducker.exe (CLI) em segundo plano ocupando a porta, encerra para liberar para a UI
tasklist /FI "IMAGENAME eq ducker.exe" 2>NUL | find /I /N "ducker.exe">NUL
if "%ERRORLEVEL%"=="0" (
    echo Liberando porta para a interface grafica...
    taskkill /IM ducker.exe /F >nul 2>nul
    timeout /t 1 /nobreak >nul
)

:: Se houver ducker-app.exe preso em segundo plano sem janela visivel, encerra antes de abrir
tasklist /FI "IMAGENAME eq ducker-app.exe" 2>NUL | find /I /N "ducker-app.exe">NUL
if "%ERRORLEVEL%"=="0" (
    echo Reiniciando instancia anterior do Ducker App...
    taskkill /IM ducker-app.exe /F >nul 2>nul
    timeout /t 1 /nobreak >nul
 )

if exist "%~dp0target\debug\ducker-app.exe" (
    echo Abrindo o Ducker App...
    start "" "%~dp0target\debug\ducker-app.exe" %*
) else (
    echo Compilando e abrindo o Ducker...
    cargo run -p ducker-app
)

