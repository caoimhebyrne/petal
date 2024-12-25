#!/bin/sh

set -e # If an error occurrs in any command, stop execution.

BIN_DIRECTORY="$HOME/.local/bin"
SOURCE_PATH="./build/petal"
DESTINATION_PATH="$BIN_DIRECTORY/petal"

# Prompts the user for a boolean response given a certain message.
# Returns: 0 if yes, 1 if no.
function prompt_boolean() {
    printf "$1 (y/n)? "
    read answer

    if [ "$answer" != "${answer#[Yy]}" ]; then
        return 1 # No
    else
        return 0 # Yes
    fi
}

# This script is a quick and dirty script for installing the Petal compiler.
# It will only be installed for your user, not system-wide.
if prompt_boolean "Do you want to build and install the Petal compiler to $DESTINATION_PATH"; then
    exit -1
fi

# Before installing, check if a file already exists at the destination path.
if [ -x "$DESTINATION_PATH" ]; then
    if prompt_boolean "A file already exists at $DESTINATION_PATH, override"; then
        exit -1
    fi
fi

# Build the compiler, ensuring that only errors are printed.
echo "+ make build"
make build > /dev/null

# Move the compiler to the installation path.
echo "+ cp $SOURCE_PATH $DESTINATION_PATH"
cp $SOURCE_PATH $DESTINATION_PATH

echo
echo "Done! The Petal compiler is now available at $DESTINATION_PATH."

if [ ! -x $(which petal) ]; then
    echo
    echo "Warning: $BIN_DIRECTORY may not be on your PATH."
    echo "To fix this, add \`export \$PATH=\$PATH:$BIN_DIRECTORY\` to your shell's RC file (.bashrc, .zshrc, etc.)"
fi
