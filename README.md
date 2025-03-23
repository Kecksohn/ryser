# ryser

Library Browser for Films & TV Shows using Rust, Tauri & React

A project born from the fact that mpc-hc lags a BIT when used with Kodi. Will hopefully do all of Kodi's functionalities I care about & be prettier too. Also my first project in Rust.

# Download Latest

yeah yeah soon chill u crazy shit

# Dev Set-Up

Get [Nodejs >= 22](https://github.com/nvm-sh/nvm/) ([Windows](https://github.com/coreybutler/nvm-windows/)), [yarn](https://classic.yarnpkg.com/lang/en/docs/install/#windows-stable) and [Rust](https://www.rust-lang.org/tools/install).

Follow [this set-up](https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building) to successfully build ffmpeg-next (rust-ffmpeg) on your platform.
<details><summary>or follow these ffmpeg-next Windows Build Instructions</summary>

* [Install LLVM](https://releases.llvm.org/download.html) (LLVM-xx.x.x-win64.exe on linked GitHub release)
* [Download FFMPEG >=7.1.1](https://github.com/GyanD/codexffmpeg/releases) (choose full-build-shared! or similar, if unsure [check here](https://ffmpeg.org/download.html) or [just download this](https://github.com/GyanD/codexffmpeg/releases/download/7.1.1/ffmpeg-7.1.1-full_build-shared.7z)) \
    (click on one of the windows build links, then look for an archive that's tagged with 'shared', 'full' or 'w64' also don't hurt; use your head) \
    (the archive should contain at least a /bin/ an /include/ and a /lib/ folder)
* Add both LLVM's and FFMPEG's `bin` folders to your `PATH`
* Create `FFMPEG_DIR` environmental variable and set it to your extracted FFMPEG dir (where include and lib reside)
* Restart your shell or PC (verify installs & paths using `clang -v` and `ffmpeg -version`) 
</details>

</br>

Run the following commands:
```
yarn
yarn tauri dev
```
<details><summary>If building fails due to ffmpeg looking for vcpkg even though you installed it as above</summary>
```
git clone https://github.com/microsoft/vcpkg
cd vcpkg
vcpkg integrate install
```
(Restart shell or PC and try again)
</details>

<details><summary>If building fails due to ffmpeg failing to compile with missing sys::AVCodecID or similar</summary>
Try

```
yarn tauri build
```

If that works you can just run the dev build as seperate components by opening one shell with ```yarn run``` and one with ```cargo run``` (see also the RustRover Debug set-up below)
</details>

### Debugging With RustRover
Open the folder, if it asks for attaching cargo and you don't know what that is click Attach.

Create 2 Configurations:
- npm -> Scripts: dev
- Cargo -> Command: run --no-default-features

Start the server using the npm script, then launch the cargo run config using the debugger.

### Faster Compilation

This is Rust so you pretty much take what you can get. However, if you don't need the GUI and just wanna test some functions in the back-end you can run:

```cargo run --features debug-backend```

Which skips building the front-end (approximately x3 the compile time) and executes whatever you put in ```debug_main()``` inside ```/src-tauri/src/_debug_run/```, after the usual initialization.

# Dev Build

```
yarn tauri build
```
