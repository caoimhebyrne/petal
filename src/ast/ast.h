#ifndef __AST_H__
#define __AST_H__

#include "../diagnostics.h"
#include "../lexer/token.h"
#include "node.h"
#include "node/function_declaration.h"
#include "node/variable_declaration.h"

typedef struct {
    // The token stream to parse from.
    TokenStream token_stream;

    // The diagnostic stream associated with this parsing session.
    DiagnosticStream diagnostics;

    // The index of the token that the parser is currently parsing.
    size_t position;
} AST;

// Creates an AST parser with a token stream.
// Parameters:
// - ast: The AST parser to initialize.
// - token_stream: The token stream to parse from.
bool ast_initialize(AST* ast, TokenStream token_stream);

// De-allocates the contents held within the provided AST.
// Parameters:
// - ast: The AST to destroy.
void ast_destroy(AST* lexer);

// Parses a node stream from the data associated with the AST.
// If the AST's diagnostics length is greater than zero, this node stream is incomplete.
// Parameters:
// - ast: The AST to start parsing.
NodeStream ast_parse(AST* ast);

// Expects a certain token to be at the current position in the token stream.
// If the token is not present, a diagnostic is emitted and an invalid token is returned.
// If the token is present, the position is advanced, and the token is returned.
Token ast_expect_token(AST* ast, TokenType type);

// Parses a statement node at the current position.
// Parameters:
// - ast: The AST to use when parsing.
// Returns:
// - A node if successful, otherwise 0.
Node* ast_parse_statement(AST* ast);

// Parses a value node at the current position.
// Parameters:
// - ast: The AST to use when parsing.
// Returns:
// - A node if successful, otherwise 0.
Node* ast_parse_value(AST* ast);

// Parses a variable declaration node at the current position.
// Parameters:
// - ast: The AST to use when parsing.
// Returns:
// - A variable declaration node if successful, otherwise 0.
VariableDeclarationNode* ast_parse_variable_declaration(AST* ast);

// Parses a function declaration node at the current position.
// Parameters:
// - ast: The AST to use when parsing.
// Returns:
// - A function declaration node if successful, otherwise 0.
FunctionDeclarationNode* ast_parse_function_declaration(AST* ast);

#endif // __AST_H__
