#include "ast.h"
#include "allocator.h"
#include "array.h"
#include "lexer.h"
#include "logger.h"
#include <assert.h>

IMPLEMENT_ARRAY_TYPE(NodeArray, node_array, Node*)

Node* function_declaration_node_create(Allocator* allocator, StringBuffer name) {
    Node* node = allocator_alloc(allocator, sizeof(Node));
    assert(node && "Failed to allocate function declaration node");

    node->function_declaration = (FunctionDeclarationNode){.name = name};

    return node;
}

// Returns whether the end of the token stream has been reached.
bool ast_parser_is_eof(const ASTParser* ast_parser) { return ast_parser->cursor > ast_parser->tokens->length; }

// Returns the token at the parser's current position without advancing the cursor.
const Token* ast_parser_peek(const ASTParser* ast_parser) {
    if (ast_parser_is_eof(ast_parser)) {
        return NULL;
    }

    return &ast_parser->tokens->data[ast_parser->cursor];
}

// Returns the token at the parser's current position while advancing the cursor.
const Token* ast_parser_consume(ASTParser* ast_parser) {
    if (ast_parser_is_eof(ast_parser)) {
        return NULL;
    }

    return &ast_parser->tokens->data[ast_parser->cursor++];
}

// Expects a token kind to be at the parser's current position, returning NULL if the token kind is not present.
const Token* ast_parser_expect(ASTParser* ast_parser, TokenKind kind) {
    const Token* token = ast_parser_consume(ast_parser);

    if (token == NULL || token->kind != kind) {
        // TODO: Emit a diagnostic.
        return NULL;
    }

    return token;
}

void ast_parser_init(ASTParser* ast_parser, Allocator* allocator, const TokenArray* tokens) {
    ast_parser->allocator = allocator;
    ast_parser->tokens = tokens;
    ast_parser->cursor = 0;
}

// Attempts to parse a top level statement at the AST parser's current position.
bool ast_parser_parse_top_level_statement(ASTParser* ast_parser, NodeArray* nodes);

bool ast_parser_parse(ASTParser* ast_parser, NodeArray* nodes) {
    while (!ast_parser_is_eof(ast_parser)) {
        // There should only be top-level statements at this point of the AST parser.
        if (!ast_parser_parse_top_level_statement(ast_parser, nodes)) {
            return false;
        }
    }

    return false;
}

bool ast_parser_parse_top_level_statement(ASTParser* ast_parser, NodeArray* nodes) {
    (void)ast_parser;
    (void)nodes;

    log_error("ast_parser_top_level_statement is not implemented");
    return false;
}
