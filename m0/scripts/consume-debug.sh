# Source me: consume the hgba fork's sandbox DEBUG-V8 archive (is_debug=true:
# DCHECKs, CPPGC_VERIFY_HEAP). Use a separate --target-dir (e.g. target-v8debug).
M0=/Users/joederas/Documents/Cline/code/browser/m0
unset V8_FROM_SOURCE
export V8_FORCE_DEBUG=true
export RUSTY_V8_ARCHIVE="$M0/archives/librusty_v8_ptrcomp_sandbox_debug_aarch64-apple-darwin.a"
export RUSTY_V8_SRC_BINDING_PATH="$M0/archives/src_binding_ptrcomp_sandbox_debug_aarch64-apple-darwin.rs"
export V8_CPPGC_EXT_RUSTY_V8_DIR="$M0/rusty_v8"
export CLANG_BASE_PATH="$HOME/.cache/v8-cppgc-ext/clang-llvmorg-23-init-19482-g53d18800-1"
