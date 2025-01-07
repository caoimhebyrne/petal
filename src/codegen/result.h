#pragma once

// The status of a code generation result.
typedef enum {
    CODEGEN_RESULT_SUCCESS,
    CODEGEN_RESULT_FAILURE,
} CodegenResultStatus;

// A code generation result.
typedef struct {
    // The status of this result.
    CodegenResultStatus status;
} CodegenResult;
