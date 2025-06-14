# ryser

Library Browser for Films & TV Shows using Rust, Tauri & React

A project born from the fact that mpc-hc lags a BIT when used with Kodi. Will hopefully do all of Kodi's functionalities I care about & be prettier too. Also my first project in Rust.

<br>

# Table of Contents

<!-- toc -->

- [Download Latest](#download-latest)
- [Dev Set-Up](#dev-set-up)
    + [TheMovieDatabase Integration](#themoviedatabase-integration)
    + [Debugging With RustRover](#debugging-with-rustrover)
    + [Faster Compilation](#faster-compilation)
    + [Prettier VSCode Set-Up](#prettier-vscode-set-up)
- [Dev Build](#dev-build)
- [Roadmap / TODOs](#roadmap--todos)

<!-- tocstop -->

<!-- 
TOC Generation: 

- yarn global add markdown-toc
- yarn global bin (add result to PATH)
- markdown-toc -i README.md
-->

<br>

# Download Latest

yeah yeah soon chill u crazy shit

<br>

# Dev Set-Up

Get [Nodejs >= 22](https://github.com/nvm-sh/nvm/) ([Windows](https://github.com/coreybutler/nvm-windows/)), [yarn](https://classic.yarnpkg.com/lang/en/docs/install/#windows-stable) and [Rust](https://www.rust-lang.org/tools/install).

Follow [this set-up](https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building) to successfully build ffmpeg-next (rust-ffmpeg) on your platform.

<details><summary>or follow these ffmpeg-next Windows Build Instructions</summary>

- [Install LLVM](https://releases.llvm.org/download.html) (LLVM-xx.x.x-win64.exe on linked GitHub release)
- [Download FFMPEG >=7.1.1](https://github.com/GyanD/codexffmpeg/releases) (choose full-build-shared! or similar, if unsure [check here](https://ffmpeg.org/download.html) or [just download this](https://github.com/GyanD/codexffmpeg/releases/download/7.1.1/ffmpeg-7.1.1-full_build-shared.7z))
- Add both LLVM's and FFMPEG's `bin` folders to your `PATH`
- Create `FFMPEG_DIR` environmental variable and set it to your extracted FFMPEG dir (where include and lib reside)
- Restart your shell or PC (verify installs & paths using `clang -v` and `ffmpeg -version`)
  </details>

<br/>

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

If that works you can just run the dev build as seperate components by opening one shell with `yarn dev` and one with `cargo run` (see also the RustRover Debug set-up below)

</details>

<br>

### TheMovieDatabase Integration

If you want to use TMDB you will need to get your own [API Access Token](https://www.themoviedb.org/settings/api) \
Insert it into `/src-tauri/src/library_manager/tmdb_api/api_token.rs`

After doing that you should run:

```
git update-index --skip-worktree src-tauri/src/library_manager/tmdb_api/api_token.rs
```

so you don't accidentally include it in a future pull request

<br>

### Debugging With RustRover

Open the folder, if it asks for attaching cargo and you don't know what that is click Attach.

Create 2 Configurations:

- npm -> Scripts: dev
- Cargo -> Command: run --no-default-features

Start the server using the npm script, then launch the cargo run config using the debugger.

<br>

### Faster Compilation

**1. Debug Back-End Only**

This is Rust so you pretty much take what you can get. However, if you don't need the GUI and just wanna test some functions in the back-end you can run:

`cargo run --features debug-backend`

Which skips building the front-end (approximately x3 the compile time on iterative builds) and executes whatever you put in `debug_main()` inside `/src-tauri/src/_debug_run/` after back-end initialization.

Combining this with RustRover Debugging is left as an exercise to the reader.

If you're going for 10x programming I'd recommend checking out [watchexec](https://github.com/watchexec/watchexec), which can execute the above command on every save.

**2. Stop VSCode's rust-analyzer from blocking the source directory**

By default, the build command must wait for the rust-analyzer to release its lock on the source directory.
To execute both the analyzer and your build command simultaneously, open your preffered JSON Settings using Ctrl + Shift + P (probably User Settings), then add

`"rust-analyzer.extraArgs": ["--target-dir", "C:/tmp/rust-analyzer-check"]`

Note: You may specify a different directory than "C:/tmp/rust-analyzer-check". For Linux, leave out "C:".

<br>

### Prettier VSCode Set-Up

Install the [VSCode Prettier Extension](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode)<details><summary>Enable Format on Save</summary>

`Ctrl + Shift + P` -> `Preferences: Open User Settings (JSON)`

```
  "[javascriptreact]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  },
  "[css]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  }

  // Optional, not used by the project but you might as well and enable/disable prettier per workspace
  "[javascript]": { 
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  },
  "[typescript]": { 
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  },
  "[typescriptreact]": { 
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  },
  "[json]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  },
  
```

</details>

<br>

# Dev Build

```
yarn tauri build
```

<br>

# Roadmap / TODOs

- Parse "part1.mkv" Files
- Choose Local Cover from UI
- Remember Scroll Position in Library View
- Async them start-up calls
- Refactor Rust Notifications
- Parse TV Series / Episodes
- Choose Video Player, General Settings
- Library Renaming, Settings, Default Filter/Sort, etc
- Parse Audio Languages in File, match with TMDB, choose correct on launch
- Parse Subtitle Languages, choose correct on Launch if user does not speak chosen audio
- Real-Time Library Rescan (Folder Watching)
- Start-Up Library Rescan should catch moving of whole library and other issues and ask for confirmation
