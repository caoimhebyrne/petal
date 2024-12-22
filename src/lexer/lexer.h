#ifndef __LEXER_H__
#define __LEXER_H__

#include "token.h"
#include <stdbool.h>
#include <stddef.h>

typedef struct {
    // The contents of the file to parse.
    char* contents;

    // The amount of bytes contained within `contents`.
    size_t contents_length;

    // The current index into contents that the lexer is at.
    size_t position;
} Lexer;

// Initializes the provided lexer with the contents of the provided filename.
// Parameters:
// - lexer: The lexer to initialize.
// - filename: The filename to read the contents of.
bool lexer_initialize(Lexer* lexer, char* filename);

// Iterates over the contents within the Lexer, producing a stream of tokens.
// Parameters:
// - lexer: The lexer to use when parsing.
// Returns:
// - A token stream.
//   If the length of the stream is 0, either no tokens were found, or an error occurred.
TokenStream lexer_parse(Lexer* lexer);

// Attempts to parse an identifier token from the contents at the current position in the Lexer.
// Parameters:
// - lexer: The lexer to use when parsing;
// Returns:
// - A token if an identifier could be parsed, otherwise 0.
Token lexer_parse_identifier(Lexer* lexer);

// Attempts to parse a number literal token from the contents at the current position in the Lexer.
// Parameters:
// - lexer: The lexer to use when parsing;
// Returns:
// - A token if a number literal could be parsed, otherwise 0.
Token lexer_parse_number_literal(Lexer* lexer);

// De-allocates the contents held within the provided Lexer.
// Parameters:
// - lexer: The lexer to destroy.
void lexer_destroy(Lexer* lexer);

#endif // __LEXER_H__
