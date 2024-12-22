#include "ast.h"
#include "../string/format_string.h"
#include "node.h"
#include "node/function_call.h"
#include "node/function_declaration.h"
#include "node/identifier_reference.h"
#include "node/number_literal.h"
#include "node/variable_declaration.h"
#include <string.h>

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

    while (ast->position < ast->token_stream.length) {
        Node* node = ast_parse_node(ast, true);
        if (node == 0) {
            return stream;
        }

        node_stream_append(&stream, node);
    }

    return stream;
}

Token ast_peek_token(AST* ast) { return ast->token_stream.data[ast->position]; }

Token ast_expect_token(AST* ast, TokenType type) {
    Token token = ast->token_stream.data[ast->position];
    if (token.type == TOKEN_INVALID) {
        Token last_token = ast->token_stream.data[ast->token_stream.length - 1];

        Diagnostic diagnostic = {
            .position = last_token.position,
            .message = format_string("expected %s, but got end-of-file", token_type_to_string(type)),
            .is_terminal = true,
        };

        diagnostic_stream_append(&ast->diagnostics, diagnostic);
        return INVALID_TOKEN;
    }

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

Node* ast_parse_node(AST* ast, bool as_statement) {
    Token token = ast_peek_token(ast);
    switch (token.type) {

    // In order to figure out what this identifier is for, we need to see its value, but
    // also take a look at the tokens around it.
    case TOKEN_IDENTIFIER: {
        // If this identifier is "func", the following tokens must make up a function declaration.
        if (strcmp(token.string, "func") == 0) {
            return (Node*)ast_parse_function_declaration(ast);
        }

        // To figure out what the identifier is for, we need to check the next token's value.
        Token next_token = ast->token_stream.data[ast->position + 1];
        switch (next_token.type) {
        case TOKEN_IDENTIFIER:
            // If the next token is another identifier, this *should* be a variable declaration.
            return (Node*)ast_parse_variable_declaration(ast);

        case TOKEN_OPEN_PARENTHESIS:
            // If the next token is an opening parenthesis, it's safe to say that this could be a function call.
            return (Node*)ast_parse_function_call(ast, as_statement);

        // Otherwise, the next token seems to be useless, and this is probably just an identifier reference.
        default:
            ast->position += 1;
            return (Node*)identifier_reference_node_create(token.string);
        }
    }

    case TOKEN_NUMBER_LITERAL:
        ast->position += 1;
        return (Node*)number_literal_node_create(token.number);

    case TOKEN_INVALID: {
        Token last_token = ast->token_stream.data[ast->token_stream.length - 1];

        Diagnostic diagnostic = {
            .position = last_token.position,
            .message = "expected any token, but got end-of-file",
            .is_terminal = true,
        };

        diagnostic_stream_append(&ast->diagnostics, diagnostic);
        return 0;
    }

    default:
        break;
    }

    Diagnostic diagnostic = {
        .position = token.position,
        .message = format_string("unexpected token: '%s'", token_to_string(&token)),
        .is_terminal = true,
    };

    diagnostic_stream_append(&ast->diagnostics, diagnostic);
    return 0;
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

    Node* value_node = ast_parse_node(ast, false);
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

// func <name>() { ... }
FunctionDeclarationNode* ast_parse_function_declaration(AST* ast) {
    // The first token in the stream must be an identifier which equals 'func'.
    Token func_token = ast_expect_token(ast, TOKEN_IDENTIFIER);
    if (func_token.type == TOKEN_INVALID) {
        return 0;
    }

    if (strcmp(func_token.string, "func") != 0) {
        Diagnostic diagnostic = {
            .position = func_token.position,
            .message = format_string("unexpected identifier: '%s', expected keyword 'func'", func_token.string),
            .is_terminal = true,
        };

        diagnostic_stream_append(&ast->diagnostics, diagnostic);
        return 0;
    }

    // The second token in the stream must be an identifier for the name.
    Token name_token = ast_expect_token(ast, TOKEN_IDENTIFIER);
    if (name_token.type == TOKEN_INVALID) {
        return 0;
    }

    // FIXME: There is no support for arguments/parameters in functions yet.
    Token open_parenthesis_token = ast_expect_token(ast, TOKEN_OPEN_PARENTHESIS);
    if (open_parenthesis_token.type == TOKEN_INVALID) {
        return 0;
    }

    Token close_parenthesis_token = ast_expect_token(ast, TOKEN_CLOSE_PARENTHESIS);
    if (close_parenthesis_token.type == TOKEN_INVALID) {
        return 0;
    }

    Token open_brace_token = ast_expect_token(ast, TOKEN_OPEN_BRACE);
    if (open_brace_token.type == TOKEN_INVALID) {
        return 0;
    }

    NodeStream function_body;
    node_stream_initialize(&function_body, 2);

    while (ast_peek_token(ast).type != TOKEN_CLOSE_BRACE) {
        Node* node = ast_parse_node(ast, true);
        if (node == 0) {
            return 0;
        }

        node_stream_append(&function_body, node);
    }

    Token close_brace_token = ast_expect_token(ast, TOKEN_CLOSE_BRACE);
    if (close_brace_token.type == TOKEN_INVALID) {
        return 0;
    }

    return function_declaration_node_create(name_token.string, function_body);
}

FunctionCallNode* ast_parse_function_call(AST* ast, bool as_statement) {
    // The first token in the stream must be an identifier for the name.
    Token name_token = ast_expect_token(ast, TOKEN_IDENTIFIER);
    if (name_token.type == TOKEN_INVALID) {
        return 0;
    }

    // FIXME: There is no support for arguments/parameters in functions yet.
    Token open_parenthesis_token = ast_expect_token(ast, TOKEN_OPEN_PARENTHESIS);
    if (open_parenthesis_token.type == TOKEN_INVALID) {
        return 0;
    }

    Token close_parenthesis_token = ast_expect_token(ast, TOKEN_CLOSE_PARENTHESIS);
    if (close_parenthesis_token.type == TOKEN_INVALID) {
        return 0;
    }

    if (as_statement) {
        // The last token must be a semicolon.
        Token semicolon_token = ast_expect_token(ast, TOKEN_SEMICOLON);
        if (semicolon_token.type == TOKEN_INVALID) {
            return 0;
        }
    }

    return function_call_node_create(name_token.string);
}

void ast_destroy(AST* ast) {
    token_stream_destroy(&ast->token_stream);
    diagnostic_stream_destroy(&ast->diagnostics);
}
