#ifndef __AST_H__
#define __AST_H__

#include "../diagnostics.h"
#include "../lexer/token.h"
#include "node.h"

typedef struct {
    // The token stream to consume from.
    TokenStream token_stream;

    // The position that the AST is currently at within the token stream.
    size_t position;

    // The diagnostics emitted during this parsing session.
    DiagnosticStream diagnostics;
} AST;

// Initializes an AST instance from a token stream.
// Parameters:
// - ast: The AST to initialize.
// - token_stream: The token stream to consume from.
bool ast_initialize(AST* ast, TokenStream token_stream);

// Returns the token at the current position in the AST, advancing
// the position.
// Parameters:
// - ast: The AST to consume tokens with.
// Returns:
// - The token at the current position in the AST.
//   If EOF is reached, TOKEN_INVALID is returned.
Token ast_consume(AST* ast);

// Attempts to parse an initialized AST.
// Parameters:
// - ast: The AST to use when parsing.
// Returns:
// - A node stream containing parsed AST nodes. If its length is zero,
//   the parsing may have failed. Check the `diagnostics` for terminal
//   diagnostics.
NodeStream ast_parse(AST* ast);

// Destroys an AST instance.
// Parameters:
// - ast: The AST to destroy.
void ast_destroy(AST* ast);

#endif
