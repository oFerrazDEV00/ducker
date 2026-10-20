@echo off
if exist "%~dp0target\release\ducker.exe" (
    "%~dp0target\release\ducker.exe" %*
) else if exist "%~dp0target\debug\ducker.exe" (
    "%~dp0target\debug\ducker.exe" %*
) else if exist "%USERPROFILE%\.cargo\bin\ducker.exe" (
    "%USERPROFILE%\.cargo\bin\ducker.exe" %*
) else (
    cargo run -p ducker-cli -- %*
)
