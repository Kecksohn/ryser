# ryser

A project born from the fact that mpc-hc lags a BIT when used with Kodi. Will hopefully do all of Kodi's functionalities I care about & be prettier too.
Windows only.

# Installation

NO RELEASE YET. Soon: Download the latest release and install anywhere else than Program Files.

# Set-Up

You need Qt & Boost.
- Qt6: [Install open source version via Online Installer](https://doc.qt.io/qt-6/qt-online-installation.html). Use Qt Creator for building the application if you wanna save yourself a headache. <details>
    <summary>If installed elsewhere than the default location</summary>
    
    - It's recommended to add environmental variable "Qt6_DIR" to the `{path_to_qt}\[version]\msvc2019_64\lib\cmake\Qt6`
    - Also add `msvc2019_64\bin` `msvc2019_64\plugins` and `msvc2019_64\plugins\platforms` to PATH
    - It might work without those in Qt Creator but who knows.

</details>

- Boost: [Install vcpkg](https://vcpkg.io/en/getting-started.html) & run ```.\vcpkg.exe install boost:x64-windows```

# Create an Installer

Windows:
- Build Application in Release using Qt Creator.
- Copy ONLY the built ryser.exe to `.\create_installer\packages\com.kecksolutions.ryser\data\`
- Run `windeployqt .\ryser.exe` inside above folder (if QT bins not in PATH, `{path_to_QT_dir}\[version]\msvc2019_64\bin\windeployqt.exe .\ryser.exe`)
- [Install QT Installer Framework](https://download.qt.io/official_releases/qt-installer-framework/)
- Run `{QT_INSTALLER_DIR}\bin\binarycreator.exe -c .\config\config.xml -p .\packages\ ryserinstaller-x64` inside `.\create_installer`