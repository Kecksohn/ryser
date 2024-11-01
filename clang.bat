@echo off

IF NOT EXIST out mkdir out_clang

REM Clang /nologo -j /EHsc /c /MD 
set start_time=%time%

ninja -j 12

set end_time=%time%
set /a "elapsed_hours=%end_time:~0,2% - %start_time:~0,2%"
set /a "elapsed_minutes=%end_time:~3,2% - %start_time:~3,2% + (elapsed_hours * 60)"
set /a "elapsed_seconds=%end_time:~6,2% - %start_time:~6,2% + (elapsed_minutes * 60)"
set /a "elapsed_milliseconds=%end_time:~9,2% - %start_time:~9,2%"
echo Compile Time: %elapsed_hours%h %elapsed_minutes%m %elapsed_seconds%s %elapsed_milliseconds%ms

out_clang\ryser.exe