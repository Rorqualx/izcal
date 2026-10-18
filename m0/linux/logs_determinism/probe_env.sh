# usage: source probe_env.sh release|debug  (uses the freshly rebuilt archive in target-sandbox)
export PATH=$HOME/.cargo/bin:$PATH
export RUSTUP_TOOLCHAIN=stable
unset V8_FROM_SOURCE
export CARGO_BUILD_JOBS=4
export RUSTY_V8_ARCHIVE=$HOME/hgba-m0/target-sandbox/$1/gn_out/obj/librusty_v8.a
export RUSTY_V8_SRC_BINDING_PATH=$HOME/hgba-m0/target-sandbox/$1/gn_out/src_binding.rs
export CARGO_TARGET_DIR=$HOME/hgba-m0/target-probe-$1
