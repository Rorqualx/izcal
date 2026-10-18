@echo off
set V8_FROM_SOURCE=1
set PYTHON=C:\Users\Joe\AppData\Local\Programs\Python\Python312\python.exe
set vs2022_install=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools
set WINDOWSSDKDIR=C:\hgba-m0\winsdk
set CARGO_TARGET_DIR=C:\hgba-m0\t-rel
cd /d C:\hgba-m0\probe-diag
cargo build -j 4 --release > C:\hgba-m0\logs\probe\diag_build.log 2>&1
echo EXIT %ERRORLEVEL% >> C:\hgba-m0\logs\probe\diag_build.log
copy /Y C:\hgba-m0\t-rel\release\determinism-probe.exe C:\hgba-m0\probe-diag\probe_diag.exe >> C:\hgba-m0\logs\probe\diag_build.log
