#!/bin/sh

set -e

PETAL_INSTALL_DIR=$HOME/.local/petal
PETAL_INSTALL_TEMP=$(mktemp -d)

echo "Installing to $PETAL_INSTALL_DIR, with $PETAL_INSTALL_TEMP as the temporary working directory."

pushd $PETAL_INSTALL_TEMP > /dev/null

if rustc --version > /dev/null; then
    echo "\`rustc\` is available, assuming that its version is OK (the build will fail later if not)...\n"
else
    echo "ERROR: Could not find rust nightly toolchain, run \`rustup toolchain install nightly\` before continuing."
    exit -1
fi

# 1. Download the repository to a temporary location.
echo "Downloading latest Petal source..."
curl --silent --output petal-main.zip --location "https://github.com/caoimhebyrne/petal/archive/refs/heads/main.zip"

# 2. Extract the repository.
echo "Extracting source archive..."
unzip petal-main.zip > /dev/null

# 3. Build the compiler.
pushd $PETAL_INSTALL_TEMP/petal-main > /dev/null

echo "Building the petal compiler..."
cargo build --color never -q --release

# 4. Install the compiler.
echo "Installing the compiler to $PETAL_INSTALL_DIR/bin"
mkdir -p $PETAL_INSTALL_DIR/bin
mv ./target/release/petal $PETAL_INSTALL_DIR/bin

# 5. Install the standard library.
echo "Installing the standard library to $PETAL_INSTALL_DIR/stdlib"
mkdir -p $PETAL_INSTALL_DIR/stdlib
cp -r ./stdlib/** $PETAL_INSTALL_DIR/stdlib

# 6. Done!
echo "\nDone! Petal has been installed to $PETAL_INSTALL_DIR."
echo "You will need to make the following changes to your shell environment before you will be able to use the Petal compiler:"
echo "  1. Set PETAL_STANDARD_LIBRARY_PATH to $PETAL_INSTALL_DIR/stdlib"
echo "  2. Add $PETAL_INSTALL_DIR/bin to your PATH"
echo "\nFor ZSH or Bash shells, you can add the following lines to your .zshrc or .bashrc:\n"
echo "export PETAL_STANDARD_LIBRARY_PATH=\$HOME/.local/petal/stdlib"
echo "export PATH=\$HOME/.local/petal/bin:\$PATH"

popd > /dev/null
