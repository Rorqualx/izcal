@echo off
rem Timing probes (run under an exclusive booking; "rig check" before each).
set P=target\release\cetprobe.exe
for %%p in (off-nooop on strict) do %P% launch %%p fiberbench
for %%p in (off-nooop on strict) do %P% launch %%p own
%P% broker strict-oop bench
%P% broker on-oop bench
%P% launch on-nooop regbench
%P% launch strict-nooop regbench
