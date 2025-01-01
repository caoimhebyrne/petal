#ifndef __LEXER_H__
#define __LEXER_H__

#include "core/position.h"
#include "util/file.h"
#include "util/vector.h"

// The Lexer takes an array of character and produces tokens that can be used for parsing.
typedef struct {
    // The contents to lex.
    FileContents contents;

    // The position that the lexer is currently at in the file.
    Position position;
} Lexer;

// Initializes a new Lexer.
// Parameters:
// - contents: The characters to transform into a stream of tokens.
Lexer lexer_create(FileContents contents);

// Parses the lexer's contents into a vector of tokens.
// When you are no longer using the returned vector, call token_vector_destroy.
// Parameters:
// - lexer: The lexer to use for parsing.
// Returns:
// - A pointer to a vector if successful, otherwise null.
Vector* lexer_parse(Lexer* lexer);

// Destroys a Lexer.
// Parameters:
// - lexer: The lexer to destroy.
void lexer_destroy(Lexer lexer);

#endif // __LEXER_H__
