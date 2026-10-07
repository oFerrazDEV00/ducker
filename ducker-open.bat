@echo off
if exist "%~dp0target\debug\ducker-app.exe" (
    start "" "%~dp0target\debug\ducker-app.exe" %*
) else (
    "%~dp0target\debug\ducker.exe" serve %*
)
