@echo off
rem Functional probes for Q-JS-15 (no timing). Run from this directory on the
rem CET host after "cargo build --release" and the baseline build:
rem   set CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS=-Cdebuginfo=0
rem   cargo build --release --target-dir target-nocet
set P=target\release\cetprobe.exe
ver
echo ### baseline: CETCOMPAT image, no policy attribute; then the non-CETCOMPAT image
%P% policy
target-nocet\release\cetprobe.exe policy
for %%p in (on off strict on-oop on-nooop off-nooop) do %P% launch %%p policy
echo ### runtime policy changes
%P% launch on-nooop upgrade
%P% launch off-nooop upgrade
echo ### Q2 fibers
%P% launch on fiber
%P% launch strict fibersize
echo ### Q1 own switching, forged tokens, WRSS, shadow-stack page protection
%P% launch strict own
set CETPROBE_RUNONCE=1
%P% launch strict own
set CETPROBE_RUNONCE=
%P% launch strict forge
%P% launch on forge
%P% launch off forge
%P% launch strict wrss
%P% launch strict ssprotect
echo ### Q4 segments
for %%m in (keep update) do for %%t in (basic bigframe panic backtrace thread overflow overflow-noveh overflow-pg overflow-noveh-pg) do %P% launch strict seg %%m %%t
echo ### Q5 and Q3 JIT matrix
for %%p in (on strict on-nooop strict-nooop) do for %%k in (good bad) do for %%r in (none inproc) do %P% launch %%p jit %%k %%r
%P% broker on-oop bad
%P% broker on-oop good
%P% broker on-oop late
%P% broker strict-oop good
