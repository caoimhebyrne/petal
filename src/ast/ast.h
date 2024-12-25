#ifndef __AST_H__
#define __AST_H__

#include "../diagnostics.h"
#include "../lexer/token.h"
#include "node.h"
#include "node/binary_operation.h"
#include "node/function_call.h"
#include "node/function_declaration.h"
#include "node/return.h"
#include "node/variable_declaration.h"
#include "type.h"

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

// Parses a node at the current position.
// Parameters:
// - ast: The AST to use when parsing.
// Returns:
// - A node if successful, otherwise 0.
Node* ast_parse_node(AST* ast, bool as_statement);

// Parses a Type from the current position.
// Parameters:
// - ast: The AST to use when parsing.
// Returns:
// - A Type if successful, otherwise TYPE_INVALID.
Type ast_parse_type(AST* ast);

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

// Parses a function call node at the current position.
// Parameters:
// - ast: The AST to use when parsing.
// Returns:
// - A function call node if successful, otherwise 0.
FunctionCallNode* ast_parse_function_call(AST* ast, bool as_statement);

// Parses a return statement node at the current position.
// Parameters:
// - ast: The AST to use when parsing.
// Returns:
// - A return statement node if successful, otherwise 0.
ReturnNode* ast_parse_return_statement(AST* ast);

// Parses a binary operation node at the current position.
// Parameters:
// - ast: The AST to use when parsing.
// Returns:
// - A binary operation node if successful, otherwise 0.
BinaryOperationNode* ast_parse_binary_operation(AST* ast, Node* left, Token operator_token);

#endif // __AST_H__
