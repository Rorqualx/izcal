//! Launching child processes (M1 phase E; launch design E-R2 to E-R6):
//! the per-OS launch paths behind `hg_substrate::Launcher`.

#[cfg(target_os = "linux")]
pub mod linux;
