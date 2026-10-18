# Source me. Build environment for the hgba rusty_v8 fork on macOS arm64.
M0=/Users/joederas/Documents/Cline/code/browser/m0
export V8_FROM_SOURCE=1
export PATH="$M0/tools/pybin:$PATH"
export PYTHON="$M0/tools/pybin/python3"           # arm64 Python 3.12 (system python3 here is x86_64)
export CLANG_BASE_PATH="$HOME/.cache/v8-cppgc-ext/clang-llvmorg-23-init-19482-g53d18800-1"  # = tools/clang pin
export LIBCLANG_PATH="$M0/tools/llvm21/LLVM-21.1.8-macOS-ARM64/lib"   # bindgen needs libclang >= 21.1
