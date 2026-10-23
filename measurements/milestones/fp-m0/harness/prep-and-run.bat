@echo off
rem FP-M0 Windows prep: run the Edge half first (the rig ships only Edge),
rem then try a silent per-user Firefox install (-ms, the installer's
rem documented silent switch; winget failed with 0x80070534 SID mapping
rem from the ssh service session) under a watchdog, then the Firefox half.
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0run-matrix-windows-edge.ps1"
if exist "%LOCALAPPDATA%\Mozilla Firefox\firefox.exe" goto installed
if exist "C:\Program Files\Mozilla Firefox\firefox.exe" goto installed
curl -sL -o ffsetup.exe "https://download-installer.cdn.mozilla.net/pub/firefox/releases/158.0.1/win64/en-US/Firefox%%20Setup%%20158.0.1.exe" || exit /b 1
start "" ffsetup.exe -ms
for /l %%n in (1,1,36) do (
  if exist "%LOCALAPPDATA%\Mozilla Firefox\firefox.exe" goto installed
  if exist "C:\Program Files\Mozilla Firefox\firefox.exe" goto installed
  ping -n 6 127.0.0.1 >nul
)
taskkill /f /im ffsetup.exe /im setup.exe >nul 2>nul
:installed
del ffsetup.exe 2>nul
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0run-matrix-windows-ff.ps1"
