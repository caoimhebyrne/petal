#pragma once

#include "ast/node.h"
#include "core/diagnostic.h"
#include "lexer/token.h"

typedef struct {
    // The vector of tokens to parse.
    // This will be destroyed when `ast_destroy` is called.
    TokenVector tokens;

    // A reference to a vector of diagnostics.
    DiagnosticVector* diagnostics;

    // The current index that the AST parser is at within the token vector.
    size_t position;
} AST;

// Initializes a new AST parser.
// Parameters:
// - diagnostics: A reference to a vector of diagnostics.
// - tokens: A vector of tokens to use during parsing.
AST ast_create(DiagnosticVector* diagnostics, TokenVector tokens);

// Parses a vector of nodes.
// Parameters:
// - ast: The AST instance to use during parsing.
NodeVector ast_parse(AST* ast);

// De-allocates the data held by an AST.
// Parameters:
// - ast: The AST to destroy.
void ast_destroy(AST ast);
