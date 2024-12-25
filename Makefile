CC = clang
CFLAGS = -Wall -Wextra -Werror -Wno-pointer-to-int-cast -DDEBUG $(shell llvm-config --cflags --ldflags --system-libs --libs core)
INSTALL_DIR = $(HOME)/.local/bin

prepare:
	mkdir -p build

build: prepare
	$(CC) $(LDFLAGS) $(CFLAGS) ./src/ast/node/binary_operation.c ./src/ast/node/function_call.c ./src/ast/node/function_declaration.c ./src/ast/node/identifier_reference.c ./src/ast/node/number_literal.c ./src/ast/node/return.c ./src/ast/node/variable_declaration.c ./src/ast/ast.c ./src/ast/node.c ./src/ast/parameter.c ./src/ast/type.c ./src/codegen/llvm_codegen.c ./src/codegen/stored_values.c ./src/lexer/lexer.c ./src/lexer/token.c ./src/string/string_builder.c ./src/arguments.c ./src/diagnostics.c ./src/main.c ./src/position.c -o ./build/petal

run: build
	./build/petal

.PHONY: install
install: build
	install -Dm755 ./build/petal $(INSTALL_DIR)/petal
