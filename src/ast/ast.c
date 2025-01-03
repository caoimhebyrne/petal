#include "ast/ast.h"
#include "ast/node.h"
#include "ast/node/identifier_reference.h"
#include "ast/node/variable_declaration.h"
#include "core/type.h"
#include "lexer/token.h"
#include "util/vector.h"
#include <stdio.h>
#include <string.h>

// Forward declarations:
Node* ast_parse_statement(AST* ast);
Node* ast_parse_variable_declaration(AST* ast);

Node* ast_parse_value(AST* ast);
Node* ast_parse_identifier_reference(AST* ast);

AST ast_create(TokenVector tokens) {
    return (AST){
        .tokens = tokens,
        .position = 0,
    };
}

Token ast_peek(AST* ast) {
    if (ast->position >= ast->tokens.length) {
        return TOKEN_INVALID;
    }

    return vector_get(ast->tokens, ast->position);
}

Token ast_consume_type(AST* ast, TokenType token_type) {
    auto token = ast_peek(ast);
    if (token.type == token_type) {
        ast->position++;
        return token;
    }

    return TOKEN_INVALID;
}

bool ast_next_is(AST* ast, TokenType token_type) {
    return ast_peek(ast).type == token_type;
}

bool ast_after_next_is(AST* ast, TokenType token_type) {
    // If the requested index is outside the bounds of the vector, the token at that index is not valid.
    auto index = ast->position + 1;
    if (index >= ast->tokens.length) {
        return false;
    }

    return vector_get(ast->tokens, index).type == token_type;
}

NodeVector ast_parse(AST* ast) {
    NodeVector vector = vector_create();
    if (!vector_initialize(vector, 1)) {
        return vector;
    }

    // Keep consuming tokens until there are none left.
    while (ast->position < ast->tokens.length) {
        auto statement = ast_parse_statement(ast);
        if (!statement) {
            break;
        }

        vector_append(vector, statement);
    }

    return vector;
}

Node* ast_parse_statement(AST* ast) {
    Node* statement = nullptr;

    if (ast_next_is(ast, TOKEN_TYPE_IDENTIFIER) && ast_after_next_is(ast, TOKEN_TYPE_IDENTIFIER))
        statement = ast_parse_variable_declaration(ast);

    // If a statement could not be parsed, bail out early.
    if (statement == nullptr) {
        return nullptr;
    }

    // All statements must end in a semicolon.
    Token semicolon_token = ast_consume_type(ast, TOKEN_TYPE_SEMICOLON);
    if (semicolon_token.type == TOKEN_TYPE_INVALID) {
        // The statement was still parsed, it is just invalid, we should destroy it to prevent memory leaks.
        node_destroy(statement);
        return nullptr;
    }

    return statement;
}

// <identifier> <identifier> = (value)
Node* ast_parse_variable_declaration(AST* ast) {
    // The first token must be an identifier, this is the type.
    auto type_token = ast_consume_type(ast, TOKEN_TYPE_IDENTIFIER);
    if (type_token.type == TOKEN_TYPE_INVALID) {
        return nullptr;
    }

    // The second token must be an identifier, this is the name.
    auto name_token = ast_consume_type(ast, TOKEN_TYPE_IDENTIFIER);
    if (name_token.type == TOKEN_TYPE_INVALID) {
        return nullptr;
    }

    // The next token must be an equals.
    auto equals_token = ast_consume_type(ast, TOKEN_TYPE_EQUALS);
    if (equals_token.type == TOKEN_TYPE_INVALID) {
        return nullptr;
    }

    // The next token(s) must be the value.
    auto value = ast_parse_value(ast);
    if (!value) {
        return nullptr;
    }

    auto type = (Type*)type_create_unresolved(strdup(type_token.string));
    if (!type) {
        return nullptr;
    }

    return (Node*)variable_declaration_node_create(equals_token.position, type, strdup(name_token.string), value);
}

Node* ast_parse_value(AST* ast) {
    if (ast_next_is(ast, TOKEN_TYPE_IDENTIFIER))
        return ast_parse_identifier_reference(ast);

    fprintf(stderr, "unexpected token: %d\n", ast_peek(ast).type);
    return nullptr;
}

Node* ast_parse_identifier_reference(AST* ast) {
    // The first token must be an identifier.
    auto identifier_token = ast_consume_type(ast, TOKEN_TYPE_IDENTIFIER);
    if (identifier_token.type == TOKEN_TYPE_INVALID) {
        return nullptr;
    }

    return (Node*)identifier_reference_node_create(identifier_token.position, strdup(identifier_token.string));
}

void ast_destroy(AST ast) {
    vector_destroy(ast.tokens, token_destroy);
}
