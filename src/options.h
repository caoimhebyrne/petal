#pragma once

typedef struct {
    // Whether the help menu should be displayed.
    bool display_help;

    // The expected output binary name, may be null.
    char* output_binary_name;
} ProgramOptions;
