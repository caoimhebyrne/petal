#ifndef __ARGUMENTS_H__
#define __ARGUMENTS_H__

#include "stream.h"
#include <stddef.h>

DECLARE_STREAM(ExtraArguments, extra_arguments, char*);

typedef enum {
    // An argument where a string value is expected.
    ARGUMENT_TYPE_STRING,

    // An argument where a boolean value is expected.
    // This value is produced by the flag existing or not.
    ARGUMENT_TYPE_FLAG,
} ArgumentType;

typedef struct {
    // The name of this argument, must not be null.
    char* name;

    // The short name of this argument, can be null.
    char short_name;

    // The type that this argument's value should be, must not be null.
    ArgumentType type;

    // A simple help message associated with this argument, must not be null.
    char* message;

    // A pointer to the variable to store this argument's value in, may be 0.
    void* value;
} Argument;

// Attempts to parse values for the provided arguments from argc & argv.
// Parameters:
// - argc: The amount of elements within argv.
// - argv: The arguments passed to this program.
// - arguments: The arguments to parse values for.
// - arguments_length: The number of Arguments in arguments.
// - extra_arguments: A dynamic array to store extra arguments in.
void parse_arguments(
    size_t argc,
    char** argv,
    Argument* arguments,
    size_t arguments_length,
    ExtraArguments* extra_arguments
);

// Prints a help message to stderr with the arguments that this program takes.
// Parameters:
// - executable_name: The name of this executable, typically from `argv[0]`.
// - arguments: The arguments to print.
// - arguments_length: The number of Arguments in arguments.
void print_help_message(char* executable_name, Argument* arguments, size_t arguments_length);

#endif // __ARGUMENTS_H__
