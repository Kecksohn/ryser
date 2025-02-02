# ryser

A project born from the fact that mpc-hc lags a BIT when used with Kodi. Will hopefully do all of Kodi's functionalities I care about & be prettier too. Also my first project in Rust.

Using Tauri, Rust & React

# Installation

NO RELEASE YET. Soon: Download the latest release and install anywhere other than Program Files.

# Dev Set-Up

Get Nodejs >= 20, npm, rust and yarn.
```
yarn
yarn tauri dev
```

## Debugging With RustRover
Create 2 Configurations:
- npm -> Scripts: dev
- Cargo -> Command: run --no-default-features

Start the server using the npm script, then launch the cargo run config using the debugger.

# Dev Build

```
yarn tauri build
```