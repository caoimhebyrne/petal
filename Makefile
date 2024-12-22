CC = gcc
CFLAGS = -Wall -Wextra -Werror -DDEBUG -ggdb3 -Og

prepare:
	mkdir -p build

build: prepare
	$(CC) $(CFLAGS) ./src/ast/ast.c ./src/lexer/lexer.c ./src/lexer/token.c ./src/string/string_builder.c ./src/diagnostics.c ./src/main.c -o ./build/main

run: build
	./build/main
