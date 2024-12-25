#include "ast.h"
#include "../string/format_string.h"
#include "node.h"
#include "node/binary_operation.h"
#include "node/function_call.h"
#include "node/function_declaration.h"
#include "node/identifier_reference.h"
#include "node/number_literal.h"
#include "node/return.h"
#include "node/variable_declaration.h"
#include "parameter.h"
#include "type.h"
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

Token ast_consume_token(AST* ast) { return ast->token_stream.data[ast->position++]; }

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

Token ast_expect_keyword(AST* ast, char* keyword) {
    Token token = ast_expect_token(ast, TOKEN_KEYWORD);
    if (token.type == TOKEN_INVALID) {
        return INVALID_TOKEN
    }

    if (strcmp(token.string, keyword) != 0) {
        Diagnostic diagnostic = {
            .position = token.position,
            .message = format_string("unexpected keyword: '%s', expected: '%s'", token.string, keyword),
            .is_terminal = true,
        };

        diagnostic_stream_append(&ast->diagnostics, diagnostic);
        return INVALID_TOKEN;
    }

    return token;
}

Node* ast_parse_node(AST* ast, bool as_statement) {
    Token token = ast_peek_token(ast);
    Node* node;

    switch (token.type) {

    case TOKEN_KEYWORD: {
        // There are multiple identifiers that can indicate a function definition.
        // - func
        // - extern
        // If this identifier matches one of thse, the following tokens must make up a function declaration.
        if (strcmp(token.string, "func") == 0 || strcmp(token.string, "extern") == 0) {
            node = (Node*)ast_parse_function_declaration(ast);
        }

        // If this identifier is "return", then this is a return statement.
        if (strcmp(token.string, "return") == 0) {
            node = (Node*)ast_parse_return_statement(ast);
        }

        break;
    }

    // In order to figure out what this identifier is for, we need to see its value, but
    // also take a look at the tokens around it.
    case TOKEN_IDENTIFIER: {
        // To figure out what the identifier is for, we need to check the next token's value.
        Token next_token = ast->token_stream.data[ast->position + 1];
        switch (next_token.type) {
        case TOKEN_IDENTIFIER:
            // If the next token is another identifier, this *should* be a variable declaration.
            node = (Node*)ast_parse_variable_declaration(ast);
            break;

        case TOKEN_OPEN_PARENTHESIS:
            // If the next token is an opening parenthesis, it's safe to say that this could be a function call.
            node = (Node*)ast_parse_function_call(ast, as_statement);
            break;

        // Otherwise, the next token seems to be useless, and this is probably just an identifier reference.
        default:
            ast->position += 1;
            node = (Node*)identifier_reference_node_create(token.position, token.string);
            break;
        }

        break;
    }

    case TOKEN_NUMBER_LITERAL:
        ast->position += 1;
        node = (Node*)number_literal_node_create(token.position, token.number);
        break;

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

    if (node == 0) {
        return 0;
    }

    // Before parsing the current node, we should lookahead to see if an operator is used after this node.
    Token next_token = ast_peek_token(ast);
    if (next_token.type == TOKEN_PLUS || next_token.type == TOKEN_HYPHEN || next_token.type == TOKEN_SLASH ||
        next_token.type == TOKEN_ASTERISK) {
        ast->position += 1;
        return (Node*)ast_parse_binary_operation(ast, node, next_token);
    }

    return node;
}

// <type> <name> = <value>;
VariableDeclarationNode* ast_parse_variable_declaration(AST* ast) {
    // The first token in the stream must be an identifier for the type.
    Token type_token = ast_expect_token(ast, TOKEN_IDENTIFIER);
    if (type_token.type == TOKEN_INVALID) {
        return 0;
    }

    // If this is an unsupported type, throw an error.
    Type type = type_from_string(type_token.string);
    if (type == TYPE_INVALID) {
        Diagnostic diagnostic = {
            .position = type_token.position,
            .message = format_string("unrecognized type: '%s'", type_token.string),
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

    return variable_declaration_node_create(name_token.position, name_token.string, type, value_node);
}

// <optional: modifier> func <name>(...) {...}
// OR
// extern func <name>(...);
FunctionDeclarationNode* ast_parse_function_declaration(AST* ast) {
    LOG_DEBUG("ast", "starting to parse function declaration!");

    // If the first token in the stream is "extern", we should parse this as an external function (no body).
    Token initial_token = ast_expect_token(ast, TOKEN_KEYWORD);
    if (initial_token.type == TOKEN_INVALID) {
        return 0;
    }

    LOG_DEBUG("ast", "initial token: '%s'", initial_token.string);

    // An external function has no body, just a name, parameters, and then a semicolon.
    bool is_external_function = false;

    if (strcmp(initial_token.string, "extern") == 0) {
        // The next token must be the "func" keyword.
        Token func_token = ast_expect_keyword(ast, "func");
        if (func_token.type == TOKEN_INVALID) {
            return 0;
        }

        is_external_function = true;
    } else if (strcmp(initial_token.string, "func") != 0) {
        Diagnostic diagnostic = {
            .position = initial_token.position,
            .message = format_string("unexpected keyword: '%s', expected keyword 'func'", initial_token.string),
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

    Token open_parenthesis_token = ast_expect_token(ast, TOKEN_OPEN_PARENTHESIS);
    if (open_parenthesis_token.type == TOKEN_INVALID) {
        return 0;
    }

    Parameters parameters;
    parameters_initialize(&parameters, 1);

    // The next token will indicate how we are going to parse parameters.
    // If there is a closing parenthesis, no parameters are defined.
    // In the case of an identifier, we should start parsing parameters.
    // Otherwise, throw an error, this is an unexpected token.
    Token next_token = ast_peek_token(ast);

    switch (next_token.type) {
    case TOKEN_CLOSE_PARENTHESIS:
        // There are no parameters defined.
        ast->position += 1;
        break;

    case TOKEN_IDENTIFIER:
        // This is the first parameter's name.
        while (true) {
            // The first token should be an identifier, indicating the parameter's name.
            Token parameter_name_token = ast_expect_token(ast, TOKEN_IDENTIFIER);
            if (parameter_name_token.type == TOKEN_INVALID) {
                return 0;
            }

            // The next token must be a colon.
            Token colon_token = ast_expect_token(ast, TOKEN_COLON);
            if (colon_token.type == TOKEN_INVALID) {
                return 0;
            }

            // Finally, the last token should be the parameter's type.
            Token parameter_type_token = ast_expect_token(ast, TOKEN_IDENTIFIER);
            if (parameter_type_token.type == TOKEN_INVALID) {
                return 0;
            }

            Type parameter_type = type_from_string(parameter_type_token.string);
            if (parameter_type == 0) {
                Diagnostic diagnostic = {
                    .position = parameter_type_token.position,
                    .message = format_string("unrecognized type: '%s'", parameter_type_token.string),
                    .is_terminal = true,
                };

                diagnostic_stream_append(&ast->diagnostics, diagnostic);
                return 0;
            }

            Parameter parameter = parameter_create(parameter_name_token.string, parameter_type);
            parameters_append(&parameters, parameter);

            next_token = ast_consume_token(ast);

            // If the next token is a comma, consume and continue.
            // If it is a closing parentheses, no more parsing of arguments is required.
            // Otherwise, this is an unexpected token.
            if (next_token.type == TOKEN_COMMA) {
                // no-op
            } else if (next_token.type == TOKEN_CLOSE_PARENTHESIS) {
                break;
            } else {
                Diagnostic diagnostic = {
                    .position = next_token.position,
                    .message = format_string("expected closing parentheses, but got %s", token_to_string(&next_token)),
                    .is_terminal = true,
                };

                diagnostic_stream_append(&ast->diagnostics, diagnostic);
                return 0;
            }
        }
        break;

    default: {
        Diagnostic diagnostic = {
            .position = next_token.position,
            .message = format_string("expected closing parentheses, but got %s", token_to_string(&next_token)),
            .is_terminal = true,
        };

        diagnostic_stream_append(&ast->diagnostics, diagnostic);
        return 0;
    }
    }

    // The next token should either be...
    // 1. An open brace -> This function returns `void`.
    // 2. A hyphen, this function specifies a return type.
    next_token = ast_consume_token(ast);

    Position return_type_position = next_token.position;
    char* return_type_name = "void";

    switch (next_token.type) {

    // An open brace indicates the start of the function body.
    // This should only be present on non-external functions.
    case TOKEN_OPEN_BRACE: {
        if (is_external_function) {
            Diagnostic diagnostic = {
                .position = name_token.position,
                .message = format_string("unexpected token: %s, expected semicolon", token_to_string(&next_token)),
                .is_terminal = true,
            };

            diagnostic_stream_append(&ast->diagnostics, diagnostic);
            return 0;
        }

        break;
    }

    // A semicolon indicates the end of an external function declaration.
    case TOKEN_SEMICOLON: {
        if (!is_external_function) {
            Diagnostic diagnostic = {
                .position = name_token.position,
                .message = format_string("unexpected token: semicolon, expected open brace"),
                .is_terminal = true,
            };

            diagnostic_stream_append(&ast->diagnostics, diagnostic);
            return 0;
        }

        break;
    }

    // A hyphen indicates that a return type is about to be specified (-> <type>).
    // It must be followed by a semicolon or an open brace, depending on the definition type.
    case TOKEN_HYPHEN: {
        Token angle_bracket_token = ast_expect_token(ast, TOKEN_RIGHT_ANGLE_BRACKET);
        if (angle_bracket_token.type == TOKEN_INVALID) {
            return 0;
        }

        // The next token should be an identifier indicating the return type.
        Token return_type_token = ast_expect_token(ast, TOKEN_IDENTIFIER);
        if (return_type_token.type == TOKEN_INVALID) {
            return 0;
        }

        return_type_name = return_type_token.string;
        return_type_position = return_type_token.position;

        // The next token should be an open brace or a semicolon.
        TokenType final_token_type = is_external_function ? TOKEN_SEMICOLON : TOKEN_OPEN_BRACE;
        Token final_token = ast_expect_token(ast, final_token_type);
        if (final_token.type == TOKEN_INVALID) {
            return 0;
        }

        break;
    }

    default: {
        Diagnostic diagnostic = {
            .position = name_token.position,
            .message = format_string("unexpected token: %s, expected: %s", token_to_string(&next_token),
                                     token_type_to_string(TOKEN_OPEN_BRACE)),
            .is_terminal = true,
        };

        diagnostic_stream_append(&ast->diagnostics, diagnostic);
        return 0;
    }
    }

    // If this is an unsupported type, throw an error.
    Type return_type = type_from_string(return_type_name);
    if (return_type == TYPE_INVALID) {
        Diagnostic diagnostic = {
            .position = return_type_position,
            .message = format_string("unrecognized type: '%s'", return_type_name),
            .is_terminal = true,
        };

        diagnostic_stream_append(&ast->diagnostics, diagnostic);
        return 0;
    }

    NodeStream function_body;
    node_stream_initialize(&function_body, 2);

    if (!is_external_function) {
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
    }

    return function_declaration_node_create(initial_token.position, name_token.string, parameters, return_type,
                                            function_body, is_external_function);
}

FunctionCallNode* ast_parse_function_call(AST* ast, bool as_statement) {
    // The first token in the stream must be an identifier for the name.
    Token name_token = ast_expect_token(ast, TOKEN_IDENTIFIER);
    if (name_token.type == TOKEN_INVALID) {
        return 0;
    }

    LOG_DEBUG("ast", "parsing function call for: '%s'", name_token.string);

    Token open_parenthesis_token = ast_expect_token(ast, TOKEN_OPEN_PARENTHESIS);
    if (open_parenthesis_token.type == TOKEN_INVALID) {
        return 0;
    }

    NodeStream arguments;
    node_stream_initialize(&arguments, 1);

    // If the next token is a closing parenthesis, then there are no arguments to this function.
    Token next_token = ast_peek_token(ast);
    if (next_token.type == TOKEN_CLOSE_PARENTHESIS) {
        ast->position += 1;
    } else {
        while (true) {
            // Attempt to parse a value.
            Node* argument = ast_parse_node(ast, false);
            if (argument == 0) {
                return 0;
            }

            node_stream_append(&arguments, argument);

            // The next token will indicate how we are going to parse arguments.
            // If there is a closing parenthesis, no arguments are defined.
            Token next_token = ast_peek_token(ast);
            if (next_token.type == TOKEN_CLOSE_PARENTHESIS) {
                ast->position += 1;
                break;
            }

            // If the next token is a comma, just continue.
            if (next_token.type == TOKEN_COMMA) {
                ast->position += 1;
                continue;
            }
        }
    }

    if (as_statement) {
        // The last token must be a semicolon.
        Token semicolon_token = ast_expect_token(ast, TOKEN_SEMICOLON);
        if (semicolon_token.type == TOKEN_INVALID) {
            return 0;
        }
    }

    return function_call_node_create(name_token.position, name_token.string, arguments);
}

// return <value>;
ReturnNode* ast_parse_return_statement(AST* ast) {
    // The first token in the stream must be the "return" keyword.
    Token return_token = ast_expect_keyword(ast, "return");
    if (return_token.type == TOKEN_INVALID) {
        return 0;
    }

    // If the next token is a semicolon, there should not be a value.
    Token next_token = ast_peek_token(ast);

    if (next_token.type == TOKEN_SEMICOLON) {
        ast->position += 1;
        return return_node_create(return_token.position, 0);
    } else {
        Node* value = ast_parse_node(ast, false);
        if (!value) {
            return 0;
        }

        // The last token must be a semicolon.
        Token semicolon_token = ast_expect_token(ast, TOKEN_SEMICOLON);
        if (semicolon_token.type == TOKEN_INVALID) {
            return 0;
        }

        return return_node_create(return_token.position, value);
    }
}

BinaryOperationNode* ast_parse_binary_operation(AST* ast, Node* left, Token operator_token) {
    LOG_DEBUG("ast", "parsing binary operation");

    Operator operator_;
    switch (operator_token.type) {
    case TOKEN_PLUS:
        operator_ = OPERATOR_PLUS;
        break;

    case TOKEN_HYPHEN:
        operator_ = OPERATOR_MINUS;
        break;

    case TOKEN_SLASH:
        operator_ = OPERATOR_DIVIDE;
        break;

    case TOKEN_ASTERISK:
        operator_ = OPERATOR_MULTIPLY;
        break;

    default: {
        Diagnostic diagnostic = {
            .position = operator_token.position,
            .message = format_string("unexpected token: '%s', expected an operator", token_to_string(&operator_token)),
            .is_terminal = true,
        };

        diagnostic_stream_append(&ast->diagnostics, diagnostic);
        return 0;
    }
    }

    Node* right = ast_parse_node(ast, false);
    if (!right) {
        return 0;
    }

    BinaryOperationNode* node = binary_operation_node_create(operator_token.position, left, right, operator_);

    LOG_DEBUG("ast", "parsed binary operation: '%s'", binary_operation_node_to_string(node));
    return node;
}

void ast_destroy(AST* ast) {
    token_stream_destroy(&ast->token_stream);
    diagnostic_stream_destroy(&ast->diagnostics);
}
