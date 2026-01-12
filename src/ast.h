#pragma once

// An abstract syntax tree parser. Takes tokens from a [TokenArray] and produces a [NodeArray].
#include "allocator.h"
#include "array.h"
#include "lexer.h"

typedef enum {
    // A function declaration node.
    NODE_KIND_FUNCTION_DECLARATION,
} NodeKind;

// A function declaration node.
typedef struct {
    // The name of the function being declared.
    StringBuffer name;
} FunctionDeclarationNode;

// A node in an abstract syntax tree.
typedef struct {
    // The kind of node that this is.
    NodeKind kind;

    union {
        // Only available in `NODE_KIND_FUNCTION_DECLARATION`.
        FunctionDeclarationNode function_declaration;
    };
} Node;

DEFINE_ARRAY_TYPE(NodeArray, node_array, Node*)

// Allocates a new function declaration node with the provided allocator.
Node* function_declaration_node_create(Allocator* allocator, StringBuffer name);

typedef struct {
    // The allocator to use when allocating memory.
    Allocator* allocator;

    // The tokens to transform into AST nodes.
    const TokenArray* tokens;

    // The index into the [TokenArray] that the [ASTParser] is currently at.
    size_t cursor;
} ASTParser;

// Initializes an [ASTParser] with the provided [TokenArray].
void ast_parser_init(ASTParser* ast_parser, Allocator* allocator, const TokenArray* tokens);

// Attempts to parse an AST from the tokens in this [ASTParser].
bool ast_parser_parse(ASTParser* ast_parser, NodeArray* nodes);
