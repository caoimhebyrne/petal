# Change this to `gcc` if you would prefer to use `gcc` for building.
CC = clang

# Enable all warnings and treat them as errors.
# NOTE: To enable AddressSanitizer, add `-fsanitize=address` | `-fsanitize=undefined`.
CFLAGS = -std=c23 -I./src -Wall -Wextra -Werror -g $(shell llvm-config --cflags)

# Link with libLLVM.
LDFLAGS = $(shell llvm-config --libs --ldflags)

.PHONY: clangd
setup-clangd:
	bear --output ./build/compile_commands.json -- make build

.PHONY: prepare
prepare:
	mkdir -p build

# Compile all C files within the source directory.
SOURCES = $(shell find ./src -iname "*.c")

.PHONY: build
build: prepare
	$(CC) $(CFLAGS) $(LDFLAGS) $(SOURCES) -o ./build/petal

INSTALL_DIR = $(HOME)/.local/bin

.PHONY: install
install: build
	mkdir -p $(INSTALL_DIR)
	install -Dm755 ./build/petal $(INSTALL_DIR)/petal 
