#pragma once

#include <stddef.h>
typedef enum {
    // An argument where a string value is expected.
    ARGUMENT_KIND_STRING,

    // An argument where its presence (or lackthereof) indicates its value.
    ARGUMENT_KIND_FLAG,
} ArgumentKind;

// Represents an argument that should be parsed by the argument parser.
typedef struct {
    // The kind of this argument.
    ArgumentKind kind;

    // The name of this argument.
    // Must not be null.
    // Example: `help`.
    const char* name;

    // The "short name" for this argument.
    // May be null.
    // Example: `h`.
    const char short_name;

    // A message to be shown in the help menu for this argument.
    // Must not be null.
    // Example: "Display this message"
    const char* help_message;

    // A pointer to where the value of this argument should be stored once
    // it has been parsed.
    void* value;
} Argument;

// Attempts to parse values for the provided arguments from argc & argv.
// Parameters:
// - argc: The amount of elements within argv.
// - argv: The arguments passed to this program.
// - arguments: The arguments to parse values for.
// - arguments_length: The number of Arguments in arguments.
// - extra_argument: A pointer to store an extra argument in (if any).
void parse_arguments(size_t argc, char** argv, Argument* arguments, size_t arguments_length, char** extra_argument);

// Prints a help message to stderr with the arguments that this program takes.
// Parameters:
// - executable_name: The name of this executable, typically from `argv[0]`.
// - arguments: The arguments to print.
// - arguments_length: The number of Arguments in arguments.
void print_help_message(char* executable_name, Argument* arguments, size_t arguments_length);
