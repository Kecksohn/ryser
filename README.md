# ryser

Library Browser for Films & TV Shows using Rust, Tauri & React

A project born from the fact that mpc-hc lags a BIT when used with Kodi. Will hopefully do all of Kodi's functionalities I care about & be prettier too. Also my first project in Rust.

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

**2. Watchexec and VSCode's rust-analyzer integration**

rust-analyzer blocks the directory on save, leading to your build taking longer than it needs.
Unfortunately, old workarounds using --target-dir no longer work, so either you use
`"rust-analyzer.cargo.extraEnv": {"CARGO": script.bat/sh}` to a wrapper that calls `cargo --target-dir path` (untested!) or you disable it when using watchexec by

Ctrl + Shift + P -> Open User Settings (JSON) ->
`"rust-analyzer.checkOnSave.enable: false"`

<details><summary>Old workaround ;__;</summary>
By default, the build command must wait for the rust-analyzer to release its lock on the source directory.
To execute both the analyzer and your build command simultaneously, open your preffered JSON Settings using Ctrl + Shift + P, then add

`"rust-analyzer.extraArgs": ["--target-dir", "C:/tmp/rust-analyzer-check"]`

Note: You may specify a different directory than "C:/tmp/rust-analyzer-check". For Linux, leave out "C:".

</details>

<br>

# Dev Build

```
yarn tauri build
```

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
- Put this somewhere else in the Readme after merging:
  ```
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true
  },
  ```
