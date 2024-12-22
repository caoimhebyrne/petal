#include "ast.h"
#include "../string/format_string.h"
#include "node.h"
#include "node/number_literal.h"
#include "node/variable_declaration.h"

bool ast_initialize(AST* ast, TokenStream token_stream) {
    ast->diagnostics = (DiagnosticStream){};
    ast->token_stream = token_stream;
    ast->position = 0;

    return diagnostic_stream_initialize(&ast->diagnostics, 2);
}

NodeStream ast_parse(AST* ast) {
    NodeStream stream;
    if (!node_stream_initialize(&stream, 2)) {
        return stream;
    }

    for (; ast->position < ast->token_stream.length; ast->position++) {
        Node* node = ast_parse_statement(ast);
        if (node == 0) {
            return stream;
        }

        node_stream_append(&stream, node);
    }

    return stream;
}

Token ast_expect_token(AST* ast, TokenType type) {
    Token token = ast->token_stream.data[ast->position];
    if (token.type != type) {
        Diagnostic diagnostic = {
            .position = token.position,
            .message = format_string("unexpected token: %s, expected: %s", token_to_string(&token),
                                     token_type_to_string(type)),
            .is_terminal = true,
        };

        diagnostic_stream_append(&ast->diagnostics, diagnostic);
        return INVALID_TOKEN;
    }

    ast->position++;
    return token;
}

Node* ast_parse_statement(AST* ast) {
    Token token = ast->token_stream.data[ast->position];

    switch (token.type) {
    // The only token type supported for statements is an identifier.
    // This is because there is only one statement, a variable declaration.
    case TOKEN_IDENTIFIER:
        return (Node*)ast_parse_variable_declaration(ast);

    default: {
        Diagnostic diagnostic = {
            .position = token.position,
            .message = format_string("unexpected token: '%s'", token_to_string(&token)),
            .is_terminal = true,
        };

        diagnostic_stream_append(&ast->diagnostics, diagnostic);
        return 0;
    }
    }
}

Node* ast_parse_value(AST* ast) {
    Token token = ast_expect_token(ast, TOKEN_NUMBER_LITERAL);
    if (token.type == TOKEN_INVALID) {
        return 0;
    }

    return (Node*)number_literal_node_create(token.number);
}

// <type> <name> = <value>;
VariableDeclarationNode* ast_parse_variable_declaration(AST* ast) {
    // The first token in the stream must be an identifier for the type.
    Token type_token = ast_expect_token(ast, TOKEN_IDENTIFIER);
    if (type_token.type == TOKEN_INVALID) {
        return 0;
    }

    // The second token in the stream must be an identifier for the name.
    Token name_token = ast_expect_token(ast, TOKEN_IDENTIFIER);
    if (name_token.type == TOKEN_INVALID) {
        return 0;
    }

    // The second token in the stream must be an equals.
    Token equals_token = ast_expect_token(ast, TOKEN_EQUALS);
    if (equals_token.type == TOKEN_INVALID) {
        return 0;
    }

    Node* value_node = ast_parse_value(ast);
    if (value_node == 0) {
        return 0;
    }

    // The last token must be a semicolon.
    Token semicolon_token = ast_expect_token(ast, TOKEN_SEMICOLON);
    if (semicolon_token.type == TOKEN_INVALID) {
        return 0;
    }

    return variable_declaration_node_create(type_token.string, name_token.string, value_node);
}
