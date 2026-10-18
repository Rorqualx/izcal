@echo off
set L=C:\hgba-m0\logs\probe
mkdir %L%
set V8_FROM_SOURCE=1
set PYTHON=C:\Users\Joe\AppData\Local\Programs\Python\Python312\python.exe
set vs2022_install=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools
set WINDOWSSDKDIR=C:\hgba-m0\winsdk
cd /d C:\hgba-m0\determinism-probe
echo START %DATE% %TIME% > %L%\build_rel.log
set CARGO_TARGET_DIR=C:\hgba-m0\t-rel
cargo build -j 12 --release -vv >> %L%\build_rel.log 2>&1
echo EXIT %ERRORLEVEL% %DATE% %TIME% >> %L%\build_rel.log
echo START %DATE% %TIME% > %L%\build_dbg.log
set CARGO_TARGET_DIR=C:\hgba-m0\t-dbg
set GN_ARGS=is_debug=true
set RUSTFLAGS=-Ctarget-feature=+crt-static -Clink-arg=/NODEFAULTLIB:libcmt.lib -Clink-arg=libcmtd.lib
cargo build -j 12 --release -vv >> %L%\build_dbg.log 2>&1
echo EXIT %ERRORLEVEL% %DATE% %TIME% >> %L%\build_dbg.log
set R=C:\hgba-m0\t-rel\release\determinism-probe.exe
set D=C:\hgba-m0\t-dbg\release\determinism-probe.exe
copy /Y %R% C:\hgba-m0\determinism-probe\probe_rel.exe
copy /Y %D% C:\hgba-m0\determinism-probe\probe_dbg.exe
set R=C:\hgba-m0\determinism-probe\probe_rel.exe
set D=C:\hgba-m0\determinism-probe\probe_dbg.exe
echo %DATE% %TIME% > %L%\determinism_probe_windows.log
for %%v in (pred pred-inline pred-noflags default default-flags) do (
  %R% %%v 1000 >> %L%\determinism_probe_windows.log 2>&1
  %R% spawn %%v 50 >> %L%\determinism_probe_windows.log 2>&1
)
echo done >> %L%\determinism_probe_windows.log
echo %DATE% %TIME% > %L%\determinism_probe_windows_debugv8.log
%D% pred 200 >> %L%\determinism_probe_windows_debugv8.log 2>&1
%D% spawn pred 20 >> %L%\determinism_probe_windows_debugv8.log 2>&1
%D% pred-inline 200 >> %L%\determinism_probe_windows_debugv8.log 2>&1
%D% pred-noflags 200 >> %L%\determinism_probe_windows_debugv8.log 2>&1
%D% default-flags 200 >> %L%\determinism_probe_windows_debugv8.log 2>&1
%D% default 200 >> %L%\determinism_probe_windows_debugv8.log 2>&1
echo done >> %L%\determinism_probe_windows_debugv8.log
echo ALLDONE > %L%\all.done
