#include "arguments.h"
#include "stream.h"
#include "string/format_string.h"
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

CREATE_STREAM(ExtraArguments, extra_arguments, char*);

void parse_arguments(
    size_t argc,
    char** argv,
    Argument* arguments,
    size_t arguments_length,
    ExtraArguments* extra_arguments
) {
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

            // If the first character and the second character is a `-`, we can match it based on the long name.
            if (program_argument[0] == '-' && program_argument[1] == '-') {
                char* name = program_argument + 2;
                if (strcmp(name, argument.name) != 0) {
                    continue;
                }
            } else if (program_argument[0] == '-') {
                // If the first character is just a `-`, this should be matched based on the short name.
                if (program_argument[1] != argument.short_name) {
                    continue;
                }
            } else {
                // This is not an argument.
                break;
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

        extra_arguments_append(extra_arguments, program_argument);
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

        char* name = format_string("--%s, -%c %s", argument.name, argument.short_name, value_name);
        fprintf(stderr, "  %-30s %s\n", name, argument.message);
    }
}
