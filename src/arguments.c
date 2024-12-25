#include "arguments.h"
#include <stdbool.h>
#include <stdio.h>

void parse_arguments(size_t argc, char** argv, Argument* arguments, size_t arguments_length, char** dangling_argument) {
    if (argc == 1) {
        // The first argument is the binary filename: there are no options to parse.
        return;
    }

    // Ignore the first argument, it is the program's name.
    for (size_t i = 1; i < argc; i++) {
        char* program_argument = argv[i];
        bool did_find_matching_option = false;

        for (size_t j = 0; j < arguments_length; j++) {
            Argument argument = arguments[j];

            // If this argument does not have a `-` at the start of it, we don't care about it here.
            if (program_argument[0] != '-') {
                continue;
            }

            // If the argument's name does not match the option's name, keep searching.
            if (program_argument[1] != argument.name) {
                continue;
            }

            // If this is a flag, we do not need to check for a value.
            if (argument.type == ARGUMENT_TYPE_FLAG) {
                bool* value_pointer = (bool*)argument.value;
                *value_pointer = true;

                break;
            }

            // The next argument should be the value for this option.
            size_t value_index = i + 1;
            if (value_index > argc) {
                // This argument has no value as we have ran out of values to parse.
                break;
            }

            switch (argument.type) {
            case ARGUMENT_TYPE_FLAG: // Should not be reached.
                break;

            case ARGUMENT_TYPE_STRING: {
                char** value_pointer = (char**)argument.value;
                *value_pointer = argv[value_index];

                break;
            }
            }

            // Advancing the cursor to the value_index ensures that when this iteration is complete, the
            // value will not be parsed as an argument.
            i = value_index;

            // We can treat this argument as parsed.
            did_find_matching_option = true;
            break;
        }

        if (did_find_matching_option) {
            continue;
        }

        if (*dangling_argument == 0) {
            *dangling_argument = program_argument;
        }
    }
}

void print_help_message(char* executable_name, Argument* arguments, size_t arguments_length) {
    fprintf(stderr, "Usage: %s [options] file\n", executable_name);
    fprintf(stderr, "Options:\n");

    for (size_t i = 0; i < arguments_length; i++) {
        Argument argument = arguments[i];
        char* value_name = "";

        switch (argument.type) {
        case ARGUMENT_TYPE_STRING:
            value_name = " <string> ";
            break;
        case ARGUMENT_TYPE_FLAG:
            break;
        }

        fprintf(stderr, "  -%c%-15s%s\n", argument.name, value_name, argument.message);
    }
}
