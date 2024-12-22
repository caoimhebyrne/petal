#ifndef __AST_H__
#define __AST_H__

#include "../diagnostics.h"
#include "../lexer/token.h"

typedef struct {
    // The token stream to parse from.
    TokenStream token_stream;

    // The diagnostic stream associated with this parsing session.
    DiagnosticStream diagnostics;
} AST;

// Creates an AST parser with a token stream.
// Parameters:
// - ast: The AST parser to initialize.
// - token_stream: The token stream to parse from.
bool ast_initialize(AST* ast, TokenStream token_stream);

#endif // __AST_H__
