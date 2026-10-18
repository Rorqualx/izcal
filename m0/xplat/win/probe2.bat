@echo off
set L=C:\hgba-m0\logs\probe
set P=C:\hgba-m0\probe-diag\probe_diag.exe
set PROBE_EXIT=leak
echo %DATE% %TIME% PROBE_EXIT=leak (platform SharedRef forgotten at exit) > %L%\determinism_probe_windows_spawn_leak.log
for %%v in (pred pred-inline pred-noflags default default-flags) do (
  %P% spawn %%v 50 >> %L%\determinism_probe_windows_spawn_leak.log 2>&1
)
echo done >> %L%\determinism_probe_windows_spawn_leak.log
set PROBE_EXIT=
set V8_FROM_SOURCE=1
set PYTHON=C:\Users\Joe\AppData\Local\Programs\Python\Python312\python.exe
set vs2022_install=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools
set WINDOWSSDKDIR=C:\hgba-m0\winsdk
set RUST_BACKTRACE=1
cd /d C:\hgba-m0\rusty_v8
set CARGO_TARGET_DIR=C:\hgba-m0\t-rel
cargo test -j 12 --release --features v8_enable_sandbox --test test_predictable_platform > %L%\test_predictable_platform_rel.log 2>&1
echo EXIT %ERRORLEVEL% >> %L%\test_predictable_platform_rel.log
set CARGO_TARGET_DIR=C:\hgba-m0\t-dbg
set GN_ARGS=is_debug=true
set RUSTFLAGS=-Ctarget-feature=+crt-static -Clink-arg=/NODEFAULTLIB:libcmt.lib -Clink-arg=libcmtd.lib
cargo test -j 12 --release --features v8_enable_sandbox --test test_predictable_platform > %L%\test_predictable_platform_dbg.log 2>&1
echo EXIT %ERRORLEVEL% >> %L%\test_predictable_platform_dbg.log
echo ALLDONE > %L%\all2.done
