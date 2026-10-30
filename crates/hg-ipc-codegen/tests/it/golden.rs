//! Reserved `.hgidl` shapes other specs fix ahead of their crates, kept
//! as golden files that must keep passing the generator.

/// Enterprise policy 6.6 and EP-M0: the reserved `policy.hgidl`, copied
/// from the spec's text, passes the checker and generates. It is not in
/// `hg-ipc-gen/idl`: no crate serves it until the policy adapter exists
/// (M5 or later).
#[test]
fn the_reserved_policy_protocol_generates() {
    let src = include_str!("../golden/policy.hgidl");
    let out = hg_ipc_codegen::generate(src, "crates/hg-ipc-codegen/tests/golden/policy.hgidl")
        .unwrap_or_else(|e| panic!("policy.hgidl: {e}"));
    assert!(out.contains("PolicyError"));
}
