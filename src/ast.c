#include "ast.h"
#include "allocator.h"
#include "array.h"
#include "diagnostic.h"
#include "lexer.h"
#include <assert.h>

IMPLEMENT_ARRAY_TYPE(NodeArray, node_array, Node*)

Node* function_declaration_node_create(Allocator* allocator, StringBuffer name, NodeArray body) {
    Node* node = allocator_alloc(allocator, sizeof(Node));
    assert(node && "Failed to allocate function declaration node");

    node->function_declaration = (FunctionDeclarationNode){
        .name = name,
        .body = body,
    };

    return node;
}

// Returns whether the end of the token stream has been reached.
bool ast_parser_is_eof(const ASTParser* ast_parser) { return ast_parser->cursor >= ast_parser->tokens->length; }

// Returns the token at the parser's current position without advancing the cursor.
const Token* ast_parser_peek(const ASTParser* ast_parser) {
    if (ast_parser_is_eof(ast_parser)) {
        return NULL;
    }

    return &ast_parser->tokens->data[ast_parser->cursor];
}

// Returns whether the token at the parser's current position is of a certain kind.
bool ast_parser_peek_is(const ASTParser* ast_parser, const TokenKind kind) {
    const Token* token = ast_parser_peek(ast_parser);
    if (!token) {
        return NULL;
    }

    return token->kind == kind;
}

// Returns the token at the parser's current position while advancing the cursor.
const Token* ast_parser_consume(ASTParser* ast_parser) {
    if (ast_parser_is_eof(ast_parser)) {
        return NULL;
    }

    return &ast_parser->tokens->data[ast_parser->cursor++];
}

// Expects a token kind to be at the parser's current position, returning NULL if the token kind is not present.
const Token* ast_parser_expect(ASTParser* ast_parser, const TokenKind kind) {
    const Token* token = ast_parser_consume(ast_parser);
    if (!token || token->kind != kind) {
        // TODO: Emit a diagnostic.
        return NULL;
    }

    return token;
}

// Expects a keyword to be at the parser's current position.
bool ast_parser_expect_keyword(ASTParser* ast_parser, const Keyword keyword) {
    const Token* token = ast_parser_expect(ast_parser, TOKEN_KIND_KEYWORD);
    if (!token) {
        return false;
    }

    // TODO: Emit a diagnostic if the keyword does not match.
    return token->keyword == keyword;
}

void ast_parser_init(ASTParser* ast_parser, Allocator* allocator, DiagnosticArray* diagnostics, ModuleId module_id,
                     const TokenArray* tokens) {
    ast_parser->allocator = allocator;
    ast_parser->diagnostics = diagnostics;
    ast_parser->module_id = module_id;
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

    return true;
}

bool ast_parser_parse_statement(ASTParser* ast_parser, NodeArray* nodes) {
    (void)ast_parser;
    (void)nodes;

    diagnostic_array_append(ast_parser->diagnostics,
                            (Diagnostic){.kind = DIAGNOSTIC_KIND_ERROR,
                                         .message = "ast_parser_parse_statement is not implemented",
                                         .position = ast_parser_peek(ast_parser)->position});

    return false;
}

bool ast_parser_parse_top_level_statement(ASTParser* ast_parser, NodeArray* nodes) {
    // FIXME: This should be moved to a separate function once we have more top-level statement node kinds.
    if (!ast_parser_expect_keyword(ast_parser, KEYWORD_FUNC)) {
        return false;
    }

    const Token* name_token = ast_parser_expect(ast_parser, TOKEN_KIND_IDENTIFIER);
    if (!name_token) {
        return false;
    }

    // The next part of the declaration is the parameters, but those are not currently supported.
    if (!ast_parser_expect(ast_parser, TOKEN_KIND_OPEN_PARENTHESIS)) {
        return false;
    }

    // TODO: Parse parameters.

    if (!ast_parser_expect(ast_parser, TOKEN_KIND_CLOSE_PARENTHESIS)) {
        return false;
    }

    // Then, there may be a return type.
    if (ast_parser_peek_is(ast_parser, TOKEN_KIND_HYPHEN)) {
        if (!ast_parser_expect(ast_parser, TOKEN_KIND_HYPHEN)) {
            return false;
        }

        if (!ast_parser_expect(ast_parser, TOKEN_KIND_RIGHT_ANGLE_BRACKET)) {
            return false;
        }

        const Token* return_type_token = ast_parser_expect(ast_parser, TOKEN_KIND_IDENTIFIER);
        if (!return_type_token) {
            return false;
        }

        // TODO: Use the return type token.
    }

    if (!ast_parser_expect(ast_parser, TOKEN_KIND_OPEN_BRACE)) {
        return false;
    }

    NodeArray body = {0};
    node_array_init(&body, ast_parser->allocator);

    // We can consume the body of the function until we either reach a closing brace, or we hit the end of the file.
    while (!ast_parser_is_eof(ast_parser) && !ast_parser_peek_is(ast_parser, TOKEN_KIND_CLOSE_BRACE)) {
        if (!ast_parser_parse_statement(ast_parser, &body)) {
            return false;
        }
    }

    if (!ast_parser_expect(ast_parser, TOKEN_KIND_CLOSE_BRACE)) {
        return false;
    }

    node_array_append(nodes, function_declaration_node_create(ast_parser->allocator, name_token->string, body));
    return true;
}
