# ryser

Library Browser for Films & TV Shows using Rust, Tauri & React

A project born from the fact that mpc-hc lags a BIT when used with Kodi. Will hopefully do all of Kodi's functionalities I care about & be prettier too. Also my first project in Rust.

# Installation

yeah yeah soon chill u crazy shit

# Dev Set-Up

Get [Nodejs >= 22](https://github.com/nvm-sh/nvm/releases/]), [yarn](https://classic.yarnpkg.com/lang/en/docs/install/#windows-stable) and [Rust](https://www.rust-lang.org/tools/install).

Follow [this set-up](https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building) to successfully build ffmpeg-next (rust-ffmpeg) on your platform.
<details><summary>or follow these ffmpeg-next Windows Build Instructions</summary>

* [Install LLVM](https://releases.llvm.org/download.html) (LLVM-xx.x.x-win64.exe on linked GitHub release)
* [Download FFMPEG "full_build-shared"](https://ffmpeg.org/download.html), extract somewhere
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

### Debugging With RustRover
Create 2 Configurations:
- npm -> Scripts: dev
- Cargo -> Command: run --no-default-features

Start the server using the npm script, then launch the cargo run config using the debugger.

# Dev Build

```
yarn tauri build
```