@echo off
if exist "%~dp0target\debug\ducker.exe" (
    "%~dp0target\debug\ducker.exe" %*
) else (
    "C:\Users\gabri\.cargo\bin\ducker.exe" %*
)
