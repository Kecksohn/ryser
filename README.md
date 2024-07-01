# ryser

A project born from the fact that mpc-hc lags a BIT when used with Kodi. Will hopefully do all of Kodi's functionalities I care about & be prettier too.
Windows only.

# Installation

NO RELEASE YET. Soon: Download the latest release and install anywhere other than Program Files.

# Configuration

If you haven't installed mpc-hc to the default location or want to use something else add path and args of your video player to config.json.\
Go wild on the .css files inside /styles/

# Dev Set-Up

You need Qt & Boost.
- Qt6: [Install open source version >=6.6.3](https://www.qt.io/download-qt-installer-oss). \
Visual Studio comes with all sorts of issues when compiling Qt. If you wanna save yourself a headache use CLion or check Qt Creator in the installer and use it for building the application.
<details>
    <summary>If you do not use QtCreator or installed elsewhere than the default location</summary>
    
    - Add environmental variable "Qt6_DIR" to the `{path_to_qt}\[version]\msvc2019_64\lib\cmake\Qt6`
    - Also add `msvc2019_64\bin` `msvc2019_64\plugins` and `msvc2019_64\plugins\platforms` to PATH

</details>

- Boost: [Install vcpkg](https://vcpkg.io/en/getting-started.html) & run ```.\vcpkg.exe install boost:x64-windows```

# Create an Installer

Windows:
- [Install QT Installer Framework](https://download.qt.io/official_releases/qt-installer-framework/)
- Build Application in Release using Qt Creator.
- Copy ONLY the built ryser.exe to `.\create_installer\packages\com.kecksolutions.ryser\data\`
- Run `windeployqt .\ryser.exe` inside above folder (if QT bins not in PATH, `{path_to_QT_dir}\[version]\msvc2019_64\bin\windeployqt.exe .\ryser.exe`)
- Run `{QT_INSTALLER_DIR}\bin\binarycreator.exe -c .\config\config.xml -p .\packages\ ryserinstaller-x64` inside `.\create_installer`