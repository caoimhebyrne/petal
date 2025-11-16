# Installation

## Automatic Installation

If you are on a unix-based operating system (e.g. Linux, macOS), you can use the dedicated [install.sh](../install.sh)
script to install Petal. This will:
1. Verify that Rust is installed.
2. Download the latest Petal source code.
3. Build the Petal compiler.
4. Copy the compiler & the standard library to `~/.local/petal`.

```
curl -s https://raw.githubusercontent.com/caoimhebyrne/petal/refs/heads/main/install.sh | sh
```

## Manual Installation

### Building the compiler

1. Install the Rust nightly toolchain, it's recommended that you do this using `rustup`.
2. Clone the Petal repository.
   ```
   git clone git@github.com:caoimhebyrne/petal.git
   ```
3. Build the project in release mode.
   ```
   cargo build --release
   ```

### Installing the compiler & the standard library

1. Copy the compiler to a "global" installation directory, e.g. `~/.local/bin` (ensure that the directory is on
   your `PATH`).
   ```
   cp ./target/release/petal ~/.local/bin/petal
   ```
2. Copy the standard library to a "global" installation directory, e.g. `~/.local/petal/stdlib`.
   ```
   mkdir -p ~/.local/petal/stdlib
   cp -r ./stdlib ~/.local/petal/stdlib
   ```
3. Set the `PETAL_STANDARD_LIBRARY_PATH` environment variable in your shell's init file (e.g. `.zshrc`, `.bashrc`. etc.):
   ```
   export PETAL_STANDARD_LIBRARY_PATH=$HOME/.local/petal/stdlib
   ```
4. Done! You can now run the `petal` compiler.
