# Input options
# Each source file is assumed to be within the `src` directory
SOURCES = allocator.c array.c ast.c file.c lexer.c main.c module.c

# Compiler options
CC = gcc
CFLAGS = -Wall -Wextra -Wpedantic -I./src

# Each final object will live in the `build` directory
OBJECTS = $(addprefix ./build/,$(SOURCES:.c=.o))

# Creates the final build directory
build:
	mkdir -p ./build

# Compiles a single source file into an object file
build/%.o: ./src/%.c
	$(CC) $(CFLAGS) -c $< -o $@

# Links all object files together to make a final executable
.PHONY: petal
petal: build $(OBJECTS)
	$(CC) $(OBJECTS) -o ./build/petal

# Build everything
.PHONY: all
all: petal

# Clean up any build artifacts
.PHONY: clean
clean:
	rm -rf build

.DEFAULT_GOAL := all
