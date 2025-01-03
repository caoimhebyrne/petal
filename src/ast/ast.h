#ifndef __AST_H__
#define __AST_H__

#include "ast/node.h"
#include "lexer/token.h"

typedef struct {
    // A reference to the vector of tokens to parse.
    // This will be destroyed when `ast_destroy` is called.
    TokenVector tokens;

    // The current index that the AST parser is at within the token vector.
    size_t position;
} AST;

// Initializes a new AST parser.
// Parameters:
// - tokens: A vector of tokens to use during parsing.
AST ast_create(TokenVector tokens);

// Parses a vector of nodes.
// Parameters:
// - ast: The AST instance to use during parsing.
NodeVector ast_parse(AST* ast);

// De-allocates the data held by an AST.
// Parameters:
// - ast: The AST to destroy.
void ast_destroy(AST ast);

#endif // __AST_H__
