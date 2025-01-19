# Change this to `gcc` if you would prefer to use `gcc` for building.
CC = clang

# Enable all warnings and treat them as errors.
# NOTE: To enable AddressSanitizer, add `-fsanitize=address` | `-fsanitize=undefined`.
CFLAGS = -std=c23 -I./src -Wall -Wextra -Werror -g $(shell llvm-config --cflags)

# Link with libLLVM.
LDFLAGS = $(shell llvm-config --libs --ldflags)

# Compile all C files within the source directory.
SOURCES = $(shell find ./src -iname "*.c")
OBJECTS = $(SOURCES:./src/%.c=./build/%.o)

.PHONY: prepare
prepare:
	mkdir -p build

.PHONY: clangd
setup-clangd: prepare
	bear --output ./build/compile_commands.json -- make build

.PHONY: build
build: prepare $(OBJECTS)
	$(CC) $(LDFLAGS) $(OBJECTS) -fsanitize=address -o ./build/petal

build/%.o: src/%.c
	mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c -o $@ $<

INSTALL_DIR = $(HOME)/.local/bin

.PHONY: install
install: build
	mkdir -p $(INSTALL_DIR)
	install -Dm755 ./build/petal $(INSTALL_DIR)/petal
