VERSION=0.1.0

CC = clang
CFLAGS = -Wall -Wextra -Werror -Wno-pointer-to-int-cast -D VERSION=\"$(VERSION)\" -D DEBUG=1 $(shell llvm-config --cflags --ldflags --system-libs --libs core) -lm
INSTALL_DIR = $(HOME)/.local/bin

# All C files within the src directory.
SOURCES=$(shell find src -type f -iname '*.c')

prepare:
	mkdir -p build

build: prepare
	$(CC) $(LDFLAGS) $(CFLAGS) $(SOURCES) -o ./build/petal

run: build
	./build/petal

.PHONY: install
install: build
	install -Dm755 ./build/petal $(INSTALL_DIR)/petal

.PHONY: install-vscode-extension
install_vscode_extension:
	cp -r ./vscode-extension $(HOME)/.vscode/extensions/petal-0.1.0
