@echo off
call C:\hgba-m0\rebuild_rel.bat
call C:\hgba-m0\rebuild_dbg.bat
echo ALLDONE > C:\hgba-m0\logs\rebuild_all.done
