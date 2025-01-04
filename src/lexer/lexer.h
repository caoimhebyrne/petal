#pragma once

#include "core/diagnostic.h"
#include "core/position.h"
#include "lexer/token.h"
#include "util/file.h"

// The Lexer takes an array of character and produces tokens that can be used for parsing.
typedef struct {
    // A reference to the diagnostic vector to produce on.
    DiagnosticVector* diagnostics;

    // The contents to lex.
    FileContents contents;

    // The position that the lexer is currently at in the file.
    Position position;
} Lexer;

// Initializes a new Lexer.
// Parameters:
// - contents: The characters to transform into a stream of tokens.
Lexer lexer_create(DiagnosticVector* diagnostics, FileContents contents);

// Parses the lexer's contents into a vector of tokens.
// When you are no longer using the returned vector, call token_vector_destroy.
// Parameters:
// - lexer: The lexer to use for parsing.
// Returns:
// - A pointer to a vector if successful, otherwise null.
TokenVector lexer_parse(Lexer* lexer);

// Destroys a Lexer.
// Parameters:
// - lexer: The lexer to destroy.
void lexer_destroy(Lexer lexer);
