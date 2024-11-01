@echo off

IF NOT EXIST out mkdir out
pushd out

REM CL
REM -Oi Take sin(x) directly from CPU instead of C runtime
REM -GR- Disable C Runtime Type info (dynamic cast, etc)
REM -EHa- Disable Exceptions
REM /nologo Disable VS Print
REM /MP Multi Processor
REM /W4 Warninglevel
REM /wd4201 Disable specific Warning
REM /Od Debug Build
REM /Zi Debug Info
REM /c Compile but don't link
REM Zc:__cplusplus needed for Qt
REM /permissive- needed for Qt
REM /std:c++17 needed for Qt
set start_time=%time%
moc -o ../src/moc_HomeView.cpp ../src/QtUI/HomeView.hh && ^
moc -o ../src/moc_LibraryView.cpp ../src/QtUI/LibraryView.hh && ^
cl.exe -Oi -GR- -EHa- /nologo /MP /W4 /wd4201 /Od /Zi /c ^
    /Zc:__cplusplus /std:c++17 /permissive- ^
    /I "C:\dev\vcpkg\installed\x64-windows\include" ^
    /I "C:\dev\Qt\6.6.3\msvc2019_64\include" ^
    /I "C:\dev\Qt\6.6.3\msvc2019_64\include\QtCore" ^
    /I "C:\dev\Qt\6.6.3\msvc2019_64\include\QtGui" ^
    /I "C:\dev\Qt\6.6.3\msvc2019_64\include\QtWidgets" ^
    /I ../src/manageconfig.hh ^
    /I ../src/videoplayer.hh ^
    /I ../src/QtUI/MainWindow.hh ^
    /I ../src/QtUI/HomeView.hh ^
    /I ../src/QtUI/LibraryView.hh ^
../src/main.cpp ^
    ../src/videoplayer.cpp ^
    ../src/manageconfig.cpp ^
    ../src/QtUI/MainWindow.cpp ^
    ../src/QtUI/HomeView.cpp ^
    ../src/QtUI/LibraryView.cpp ^
    ../src/moc_HomeView.cpp ^
    ../src/moc_LibraryView.cpp

set end_time=%time%
set /a "elapsed_hours=%end_time:~0,2% - %start_time:~0,2%"
set /a "elapsed_minutes=%end_time:~3,2% - %start_time:~3,2% + (elapsed_hours * 60)"
set /a "elapsed_seconds=%end_time:~6,2% - %start_time:~6,2% + (elapsed_minutes * 60)"
set /a "elapsed_milliseconds=%end_time:~9,2% - %start_time:~9,2%"
echo Compile Time: %elapsed_hours%h %elapsed_minutes%m %elapsed_seconds%s %elapsed_milliseconds%ms



REM Linking
set start_time=%time%
ccache.exe link /nologo ^
    main.obj ^
    videoplayer.obj ^
    manageconfig.obj ^
    MainWindow.obj ^
    HomeView.obj ^
    LibraryView.obj ^
        moc_HomeView.obj ^
        moc_LibraryView.obj ^
    /LIBPATH:"C:\dev\vcpkg\installed\x64-windows\lib" ^
    /LIBPATH:"C:\dev\Qt\6.6.3\msvc2019_64\lib" ^
        Qt6Core.lib ^
        Qt6Gui.lib ^
        Qt6Widgets.lib ^
/OUT:ryser.exe

set end_time=%time%
set /a "elapsed_hours=%end_time:~0,2% - %start_time:~0,2%"
set /a "elapsed_minutes=%end_time:~3,2% - %start_time:~3,2% + (elapsed_hours * 60)"
set /a "elapsed_seconds=%end_time:~6,2% - %start_time:~6,2% + (elapsed_minutes * 60)"
set /a "elapsed_milliseconds=%end_time:~9,2% - %start_time:~9,2%"
echo Link Time: %elapsed_hours%h %elapsed_minutes%m %elapsed_seconds%s %elapsed_milliseconds%ms

ryser.exe
popd