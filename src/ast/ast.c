#include "ast/ast.h"
#include "ast/node.h"
#include "ast/node/binary_operation.h"
#include "ast/node/function_call.h"
#include "ast/node/function_declaration.h"
#include "ast/node/identifier_reference.h"
#include "ast/node/number_literal.h"
#include "ast/node/return.h"
#include "ast/node/variable_declaration.h"
#include "core/diagnostic.h"
#include "core/parameter.h"
#include "core/position.h"
#include "core/type/unresolved.h"
#include "core/type/value.h"
#include "lexer/token.h"
#include "util/defer.h"
#include "util/format.h"
#include "util/vector.h"
#include <string.h>

// Forward declarations:
Node* ast_parse_statement(AST* ast);
Node* ast_parse_variable_declaration(AST* ast);
Node* ast_parse_function_declaration(AST* ast);
Node* ast_parse_return(AST* ast);

Node* ast_parse_expression(AST* ast);
Node* ast_parse_addition_subtraction_expression(AST* ast);
Node* ast_parse_multiplication_division_expression(AST* ast);

Node* ast_parse_value(AST* ast);
Node* ast_parse_function_call(AST* ast);
Node* ast_parse_identifier_reference(AST* ast);
Node* ast_parse_number_literal(AST* ast);

void ast_diagnostic_expected_any_token(AST* ast, const char* parsing_type);

void ast_diagnostic_expected_token(AST* ast, TokenType expected, Token got);
void ast_diagnostic_internal_error(AST* ast, Position position);

AST ast_create(DiagnosticVector* diagnostics, TokenVector tokens) {
    return (AST){
        .diagnostics = diagnostics,
        .tokens = tokens,
        .position = 0,
    };
}

Position ast_last_token_position(AST* ast) {
    auto position = vector_last(ast->tokens).position;
    position.column += position.length;
    position.length = 1;

    return position;
}

Token ast_peek(AST* ast) {
    if (ast->position >= ast->tokens.length) {
        return TOKEN_INVALID;
    }

    return vector_get(&ast->tokens, ast->position);
}

Token ast_consume(AST* ast) {
    auto token = ast_peek(ast);
    if (token.type == TOKEN_TYPE_INVALID) {
        vector_append(
            ast->diagnostics,
            diagnostic_create(ast_last_token_position(ast), format_string("expected a token, but got end of file"))
        );

        return token;
    }

    ast->position++;
    return token;
}

Token ast_consume_type(AST* ast, TokenType token_type) {
    auto token = ast_peek(ast);
    if (token.type == token_type) {
        ast->position++;
        return token;
    }

    // If the token was invalid, this is the end of the file.
    if (token.type == TOKEN_TYPE_INVALID) {
        vector_append(
            ast->diagnostics,
            diagnostic_create(
                ast_last_token_position(ast),
                format_string("expected '%s', but got end of file", token_type_to_string(token_type))
            )
        );

        return token;
    }

    // Otherwise, this is just not the token type that we are looking for.
    auto token_string defer(free_str) = token_to_string(token);
    vector_append(
        ast->diagnostics,
        diagnostic_create(
            token.position,
            format_string("expected '%s', but got '%s'", token_type_to_string(token_type), token_string)
        )
    );

    return TOKEN_INVALID;
}

bool ast_next_is(AST* ast, TokenType token_type) {
    return ast_peek(ast).type == token_type;
}

bool ast_next_is_string(AST* ast, TokenType token_type, const char* value) {
    auto next = ast_peek(ast);
    if (next.type != token_type) {
        return false;
    }

    return strcmp(next.string, value) == 0;
}

bool ast_after_next_is(AST* ast, TokenType token_type) {
    // If the requested index is outside the bounds of the vector, the token at that index is not valid.
    auto index = ast->position + 1;
    if (index >= ast->tokens.length) {
        return false;
    }

    return vector_get(&ast->tokens, index).type == token_type;
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
            // Clean up the original vector as it may have items in it.
            vector_destroy(vector, node_destroy);

            // Return an invalid vector.
            return (NodeVector){};
        }

        vector_append(&vector, statement);
    }

    return vector;
}

Node* ast_parse_statement(AST* ast) {
    Node* statement = nullptr;

    if (ast_next_is(ast, TOKEN_TYPE_IDENTIFIER) && ast_after_next_is(ast, TOKEN_TYPE_IDENTIFIER))
        statement = ast_parse_variable_declaration(ast);

    else if (ast_next_is(ast, TOKEN_TYPE_IDENTIFIER) && ast_after_next_is(ast, TOKEN_TYPE_OPEN_PARENTHESIS))
        statement = ast_parse_function_call(ast);

    else if (ast_next_is_string(ast, TOKEN_TYPE_KEYWORD, "func"))
        statement = ast_parse_function_declaration(ast);

    else if (ast_next_is_string(ast, TOKEN_TYPE_KEYWORD, "return"))
        statement = ast_parse_return(ast);

    else
        ast_diagnostic_expected_any_token(ast, "statement");

    // If a statement could not be parsed, bail out early.
    if (statement == nullptr) {
        return nullptr;
    }

    // All statements must end in a semicolon - except for function declarations.
    if (statement->kind == NODE_KIND_FUNCTION_DECLARATION) {
        return statement;
    }

    auto semicolon_token = ast_consume_type(ast, TOKEN_TYPE_SEMICOLON);
    if (semicolon_token.type == TOKEN_TYPE_INVALID) {
        // The statement was still parsed, it is just invalid, we should destroy it to prevent memory leaks.
        node_destroy(statement);
        return nullptr;
    }

    return statement;
}

Type* ast_parse_type(AST* ast) {
    // A type is just an identifier at the moment, in the future it may have modifiers.
    auto type_token = ast_consume_type(ast, TOKEN_TYPE_IDENTIFIER);
    if (type_token.type == TOKEN_TYPE_INVALID) {
        return nullptr;
    }

    auto type = (Type*)unresolved_type_create(type_token.position, strdup(type_token.string));
    if (!type) {
        ast_diagnostic_internal_error(ast, type_token.position);
        return nullptr;
    }

    return type;
}

// <identifier> <identifier> = (value)
Node* ast_parse_variable_declaration(AST* ast) {
    // The first token must be an identifier, this is the type.
    auto type = ast_parse_type(ast);
    if (!type) {
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
    auto value = ast_parse_expression(ast);
    if (!value) {
        return nullptr;
    }

    return (Node*)variable_declaration_node_create(equals_token.position, type, strdup(name_token.string), value);
}

Node* ast_parse_function_declaration(AST* ast) {
    // The first token is the "func" keyword.
    auto func_token = ast_consume_type(ast, TOKEN_TYPE_KEYWORD);
    if (func_token.type == TOKEN_TYPE_INVALID) {
        return nullptr;
    }

    // The next token is an identifier for the function name.
    auto name_token = ast_consume_type(ast, TOKEN_TYPE_IDENTIFIER);
    if (name_token.type == TOKEN_TYPE_INVALID) {
        return nullptr;
    }

    // All functions must have an opening parenthesis after their name.
    // What that is followed by will help us parse the next stage.
    auto open_parenthesis_token = ast_consume_type(ast, TOKEN_TYPE_OPEN_PARENTHESIS);
    if (open_parenthesis_token.type == TOKEN_TYPE_INVALID) {
        return nullptr;
    }

    ParameterVector parameters = vector_create();
    if (!vector_initialize(parameters, 1)) {
        ast_diagnostic_internal_error(ast, open_parenthesis_token.position);
        return nullptr;
    }

    while (ast_peek(ast).type != TOKEN_TYPE_CLOSE_PARENTHESIS) {
        // Each parameter must start with a name.
        auto parameter_name_token = ast_consume_type(ast, TOKEN_TYPE_IDENTIFIER);
        if (parameter_name_token.type == TOKEN_TYPE_INVALID) {
            vector_destroy(parameters, parameter_destroy);
            return nullptr;
        }

        // After the name, there must be a colon before the parameter type.
        auto colon_token = ast_consume_type(ast, TOKEN_TYPE_COLON);
        if (colon_token.type == TOKEN_TYPE_INVALID) {
            vector_destroy(parameters, parameter_destroy);
            return nullptr;
        }

        // The next token(s) must be the parameter's type.
        Type* value_type = ast_parse_type(ast);
        if (!value_type) {
            vector_destroy(parameters, parameter_destroy);
            return nullptr;
        }

        // If the next character is not a closing parenthesis, it must be a comma.
        if (!ast_next_is(ast, TOKEN_TYPE_CLOSE_PARENTHESIS)) {
            auto comma_token = ast_consume_type(ast, TOKEN_TYPE_COMMA);
            if (comma_token.type == TOKEN_TYPE_INVALID) {
                vector_destroy(parameters, parameter_destroy);
                type_destroy(value_type);

                return nullptr;
            }
        }

        // Add the parameter to the function's parameters.
        vector_append(
            &parameters,
            parameter_create(colon_token.position, strdup(parameter_name_token.string), value_type)
        );
    }

    // After the parameters, there must be a closing parenthesis.
    auto close_parenthesis_token = ast_consume_type(ast, TOKEN_TYPE_CLOSE_PARENTHESIS);
    if (close_parenthesis_token.type == TOKEN_TYPE_INVALID) {
        vector_destroy(parameters, parameter_destroy);
        return nullptr;
    }

    // If there is a hyphen after the closing parenthesis, we should parse a return type.
    Type* return_type;
    if (ast_next_is(ast, TOKEN_TYPE_MINUS)) {
        ast_consume(ast); // Consume the minus token.

        // There must be a right angle bracket.
        auto right_angle_bracket = ast_consume_type(ast, TOKEN_TYPE_RIGHT_ANGLE_BRACKET);
        if (right_angle_bracket.type == TOKEN_TYPE_INVALID) {
            vector_destroy(parameters, parameter_destroy);
            return nullptr;
        }

        // The next token is the return type.
        return_type = ast_parse_type(ast);
    } else {
        // Otherwise, no return type was specified, let's assume void.
        return_type = (Type*)value_type_create(close_parenthesis_token.position, VALUE_TYPE_KIND_VOID);
    }

    // If a return type was not found, we can't continue.
    if (!return_type) {
        vector_destroy(parameters, parameter_destroy);
        return nullptr;
    }

    // A function's body must start with an opening brace.
    auto open_brace_token = ast_consume_type(ast, TOKEN_TYPE_OPEN_BRACE);
    if (open_brace_token.type == TOKEN_TYPE_INVALID) {
        type_destroy(return_type);
        vector_destroy(parameters, parameter_destroy);

        return nullptr;
    }

    NodeVector body = vector_create();
    if (!vector_initialize(body, 1)) {
        ast_diagnostic_internal_error(ast, open_brace_token.position);

        type_destroy(return_type);
        vector_destroy(parameters, parameter_destroy);
        return nullptr;
    }

    // Keep consuming tokens until there are none left.
    while (ast_peek(ast).type != TOKEN_TYPE_CLOSE_BRACE) {
        auto statement = ast_parse_statement(ast);
        if (!statement) {
            // Clean up the body vector as it may have items in it.
            vector_destroy(body, node_destroy);
            type_destroy(return_type);
            vector_destroy(parameters, parameter_destroy);

            return nullptr;
        }

        vector_append(&body, statement);
    }

    // All functions must end with a closing brace.
    auto close_brace_token = ast_consume_type(ast, TOKEN_TYPE_CLOSE_BRACE);
    if (close_brace_token.type == TOKEN_TYPE_INVALID) {
        vector_destroy(body, node_destroy);
        type_destroy(return_type);
        vector_destroy(parameters, parameter_destroy);

        return nullptr;
    }

    return (Node*)
        function_declaration_node_create(func_token.position, strdup(name_token.string), return_type, parameters, body);
}

Node* ast_parse_return(AST* ast) {
    // The first token is the return keyword.
    auto return_token = ast_consume_type(ast, TOKEN_TYPE_KEYWORD);
    if (return_token.type == TOKEN_TYPE_INVALID) {
        return nullptr;
    }

    // If there is a semicolon next, there is no value associated with this return.
    if (ast_next_is(ast, TOKEN_TYPE_SEMICOLON)) {
        return (Node*)return_node_create(return_token.position, nullptr);
    }

    // The next token(s) must be the value.
    auto value = ast_parse_expression(ast);
    if (!value) {
        return nullptr;
    }

    return (Node*)return_node_create(return_token.position, value);
}

Node* ast_parse_expression(AST* ast) {
    return ast_parse_addition_subtraction_expression(ast);
}

// <left> <operator> <right>
Node* ast_parse_addition_subtraction_expression(AST* ast) {
    auto left = ast_parse_multiplication_division_expression(ast);
    if (!left) {
        return nullptr;
    }

    // If the next token is plus or minus, treat this as a binary operation.
    if (ast_next_is(ast, TOKEN_TYPE_PLUS) || ast_next_is(ast, TOKEN_TYPE_MINUS)) {
        auto operator_token = ast_consume(ast);
        auto operator= operator_token.type == TOKEN_TYPE_PLUS ? OPERATOR_ADD : OPERATOR_SUBTRACT;

        auto right = ast_parse_expression(ast);
        if (!right) {
            // If we could not parse a right node, make sure to destroy the left node that was parsed.
            node_destroy(left);
            return nullptr;
        }

        return (Node*)binary_operation_node_create(operator_token.position, left, operator, right);
    }

    // There is no operator, return the left value.
    return left;
}

// <left> <operator> <right>
Node* ast_parse_multiplication_division_expression(AST* ast) {
    auto left = ast_parse_value(ast);
    if (!left) {
        return nullptr;
    }

    // If the next token is plus or minus, treat this as a binary operation.
    if (ast_next_is(ast, TOKEN_TYPE_ASTERISK) || ast_next_is(ast, TOKEN_TYPE_SLASH)) {
        auto operator_token = ast_consume(ast);
        auto operator= operator_token.type == TOKEN_TYPE_ASTERISK ? OPERATOR_MULTIPLY : OPERATOR_DIVIDE;

        auto right = ast_parse_expression(ast);
        if (!right) {
            // If we could not parse a right node, make sure to destroy the left node that was parsed.
            node_destroy(left);
            return nullptr;
        }

        return (Node*)binary_operation_node_create(operator_token.position, left, operator, right);
    }

    // There is no operator, return the left value.
    return left;
}

Node* ast_parse_value(AST* ast) {
    if (ast_next_is(ast, TOKEN_TYPE_OPEN_PARENTHESIS)) {
        ast_consume(ast);

        // Parse the value within the parenthesis.
        auto node = ast_parse_expression(ast);

        // Expect a closing parenthesis.
        auto token = ast_consume_type(ast, TOKEN_TYPE_CLOSE_PARENTHESIS);
        if (token.type == TOKEN_TYPE_INVALID) {
            return nullptr;
        }

        return node;
    }

    if (ast_next_is(ast, TOKEN_TYPE_IDENTIFIER) && ast_after_next_is(ast, TOKEN_TYPE_OPEN_PARENTHESIS))
        return ast_parse_function_call(ast);

    if (ast_next_is(ast, TOKEN_TYPE_IDENTIFIER))
        return ast_parse_identifier_reference(ast);

    if (ast_next_is(ast, TOKEN_TYPE_INTEGER_LITERAL) || ast_next_is(ast, TOKEN_TYPE_FLOAT_LITERAL))
        return ast_parse_number_literal(ast);

    ast_diagnostic_expected_any_token(ast, "value");
    return nullptr;
}

Node* ast_parse_function_call(AST* ast) {
    // The first token must be an identifier.
    auto identifier_token = ast_consume_type(ast, TOKEN_TYPE_IDENTIFIER);
    if (identifier_token.type == TOKEN_TYPE_INVALID) {
        return nullptr;
    }

    // The next token must be an opening parenthesis.
    auto open_parenthesis = ast_consume_type(ast, TOKEN_TYPE_OPEN_PARENTHESIS);
    if (open_parenthesis.type == TOKEN_TYPE_INVALID) {
        return nullptr;
    }

    // The next token(s) are the arguments to the function.
    NodeVector arguments = vector_create();
    if (!vector_initialize(arguments, 1)) {
        ast_diagnostic_internal_error(ast, open_parenthesis.position);
        return nullptr;
    }

    while (!ast_next_is(ast, TOKEN_TYPE_CLOSE_PARENTHESIS)) {
        // The current token must be a valid expression.
        auto value = ast_parse_expression(ast);
        if (!value) {
            return nullptr;
        }

        vector_append(&arguments, value);

        // If the next token is a closing parenthesis, we can break out of the loop
        if (ast_next_is(ast, TOKEN_TYPE_CLOSE_PARENTHESIS)) {
            continue;
        }

        // Otherwise, the next token must be a comma.
        auto comma = ast_consume_type(ast, TOKEN_TYPE_COMMA);
        if (comma.type == TOKEN_TYPE_INVALID) {
            return nullptr;
        }
    }

    // All function calls must end in a closing parenthesis.
    auto close_parenthesis = ast_consume_type(ast, TOKEN_TYPE_CLOSE_PARENTHESIS);
    if (close_parenthesis.type == TOKEN_TYPE_INVALID) {
        return nullptr;
    }

    return (Node*)function_call_node_create(identifier_token.position, strdup(identifier_token.string), arguments);
}

Node* ast_parse_identifier_reference(AST* ast) {
    // The first token must be an identifier.
    auto identifier_token = ast_consume_type(ast, TOKEN_TYPE_IDENTIFIER);
    if (identifier_token.type == TOKEN_TYPE_INVALID) {
        return nullptr;
    }

    return (Node*)identifier_reference_node_create(identifier_token.position, strdup(identifier_token.string));
}

Node* ast_parse_number_literal(AST* ast) {
    auto token = ast_consume(ast);

    switch (token.type) {
    case TOKEN_TYPE_INTEGER_LITERAL:
        return (Node*)number_literal_node_create_integer(token.position, token.integer);

    case TOKEN_TYPE_FLOAT_LITERAL:
        return (Node*)number_literal_node_create_float(token.position, token.number);

    default:
        auto token_string defer(free_str) = token_to_string(token);
        vector_append(
            ast->diagnostics,
            diagnostic_create(token.position, format_string("expected a number literal, but got: '%s'", token_string))
        );

        return 0;
    }
}

void ast_diagnostic_expected_any_token(AST* ast, const char* parsing_type) {
    auto current_token = ast_peek(ast);
    if (current_token.type != TOKEN_TYPE_INVALID) {
        auto token_string defer(free_str) = token_to_string(current_token);

        vector_append(
            ast->diagnostics,
            diagnostic_create(current_token.position, format_string("unexpected token: '%s'", token_string))
        );
    } else {
        vector_append(
            ast->diagnostics,
            diagnostic_create(
                ast_last_token_position(ast),
                format_string("expected a %s, but got end-of-file", parsing_type)
            )
        );
    }
}

void ast_diagnostic_internal_error(AST* ast, Position position) {
    vector_append(ast->diagnostics, diagnostic_create(position, format_string("unexpected compiler error")));
}

void ast_destroy(AST ast) {
    vector_destroy(ast.tokens, token_destroy);
}
