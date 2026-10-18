@echo off
mkdir C:\hgba-m0\logs
set LOG=C:\hgba-m0\logs\rebuild_debug_sandbox.log
set V8_FROM_SOURCE=1
set PYTHON=C:\Users\Joe\AppData\Local\Programs\Python\Python312\python.exe
set CARGO_TARGET_DIR=C:\hgba-m0\t-dbg
set PRINT_GN_ARGS=1
set GN_ARGS=is_debug=true
set RUST_BACKTRACE=1
set vs2022_install=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools
set WINDOWSSDKDIR=C:\hgba-m0\winsdk
cd /d C:\hgba-m0\rusty_v8
echo START >> %LOG%
powershell -NoProfile -c "Get-Date -Format o" >> %LOG%
cargo build -j 12 --release --features v8_enable_sandbox -vv >> %LOG% 2>&1
echo EXIT %ERRORLEVEL% >> %LOG%
powershell -NoProfile -c "Get-Date -Format o" >> %LOG%
echo DONE >> %LOG%
