#include "ast.h"
#include "node.h"
#include "node/binary_operation.h"
#include "node/block.h"
#include "node/boolean_literal.h"
#include "node/function_call.h"
#include "node/function_declaration.h"
#include "node/identifier_reference.h"
#include "node/number_literal.h"
#include "node/return.h"
#include "node/string_literal.h"
#include "node/variable_declaration.h"
#include "parameter.h"
#include "type.h"
#include <stdbool.h>
#include <string.h>

// A macro to require a token to be at the current position in the AST.
#define ast_expect(ast, token_type)                                                                                    \
    {                                                                                                                  \
        Token token = ast_consume_type(ast, token_type);                                                               \
        if (token.type == TOKEN_INVALID) {                                                                             \
            LOG_DEBUG("ast", "ast_expect! returning");                                                                 \
            return 0;                                                                                                  \
        }                                                                                                              \
    }

// Forward declarations for AST parsing methods.
// All of these return a pointer to a Node, if this pointer is zero, something
// went wrong during the parsing process.
Node* ast_parse_statement(AST* ast);
Node* ast_parse_expression(AST* ast);
Node* ast_parse_value(AST* ast);

Node* ast_parse_return_statement(AST* ast);
Node* ast_parse_variable_declaration_statement(AST* ast);
Node* ast_parse_function_declaration_statement(AST* ast);

Node* ast_parse_addition_subtraction(AST* ast);
Node* ast_parse_multiplication_division(AST* ast);
Node* ast_parse_number_literal_expression(AST* ast);
Node* ast_parse_boolean_literal_expression(AST* ast);
Node* ast_parse_string_literal_expression(AST* ast);
Node* ast_parse_identifier_reference_expression(AST* ast);

Node* ast_parse_function_call(AST* ast);

BlockNode* ast_parse_block(AST* ast);

bool ast_initialize(AST* ast, TokenStream token_stream) {
    DiagnosticStream diagnostics;
    if (!diagnostic_stream_initialize(&diagnostics, 1)) {
        return false;
    }

    ast->token_stream = token_stream;
    ast->diagnostics = diagnostics;
    ast->position = 0;

    return true;
}

Token ast_peek(AST* ast) {
    // If the AST's position is outside the bounds of the array,
    // there are no tokens left to consume.
    if (ast->position >= ast->token_stream.length) {
        return INVALID_TOKEN;
    }

    return ast->token_stream.data[ast->position];
}

Token ast_consume(AST* ast) {
    // If the AST's position is outside the bounds of the array,
    // there are no tokens left to consume.
    if (ast->position >= ast->token_stream.length) {
        return INVALID_TOKEN;
    }

    return ast->token_stream.data[ast->position++];
}

Token ast_consume_type(AST* ast, TokenType type) {
    // If the AST's position is outside the bounds of the array,
    // there are no tokens left to consume.
    if (ast->position >= ast->token_stream.length) {
        Token last_token = ast->token_stream.data[ast->token_stream.length - 1];

        diagnostic_stream_push(&ast->diagnostics, last_token.position, true,
                               "expected token: '%s', but got end of file", token_type_to_string(type));

        return INVALID_TOKEN;
    }

    Token token = ast->token_stream.data[ast->position++];
    if (token.type != type) {
        diagnostic_stream_push(&ast->diagnostics, token.position, true, "expected token: '%s', but got '%s'",
                               token_type_to_string(type), token_to_string(token));

        return INVALID_TOKEN;
    }

    return token;
}

Token ast_consume_type_string(AST* ast, TokenType type, char* string) {
    Token token = ast_consume_type(ast, type);
    if (token.type == TOKEN_INVALID) {
        return INVALID_TOKEN;
    }

    if (strcmp(token.string, string) != 0) {
        diagnostic_stream_push(&ast->diagnostics, token.position, true, "expected %s of '%s', but got '%s'",
                               token_type_to_string(token.type), string, token.string);

        return INVALID_TOKEN;
    }

    return token;
}

bool ast_next_is(AST* ast, TokenType type) {
    Token token = ast_peek(ast);
    if (token.type == TOKEN_INVALID) {
        // FIXME: This needs to be handled differently.
        // If there are no tokens left, parsing should stop.
        return false;
    }

    // Ensure the token types are matching.
    return token.type == type;
}

bool ast_after_next_is(AST* ast, TokenType type) {
    // If the intended position is out of bounds, the token will not match.
    size_t position = ast->position + 1;
    if (position >= ast->token_stream.length) {
        return false;
    }

    Token token = ast->token_stream.data[position];
    return token.type == type;
}

bool ast_next_is_string(AST* ast, TokenType type, char* string_value) {
    Token token = ast_peek(ast);
    if (token.type == TOKEN_INVALID) {
        // FIXME: This needs to be handled differently.
        // If there are no tokens left, parsing should stop.
        return false;
    }

    // Ensure the token types are matching.
    if (token.type != type) {
        return false;
    }

    // If no string value was attached to this call, just return true, they match.
    if (!string_value) {
        return true;
    }

    // Compare the two string values for a match.
    return strcmp(token.string, string_value) == 0;
}

NodeStream ast_parse(AST* ast) {
    NodeStream stream;
    node_stream_initialize(&stream, 1);

    // Keep parsing until EOF is reached.
    while (ast->position < ast->token_stream.length) {
        Node* statement = ast_parse_statement(ast);
        if (!statement) {
            // A statement could not be parsed, it's probably best to stop
            // parsing, the rest of the file is probably not usable.
            LOG_ERROR("ast", "parsing error occurred");
            break;
        }

        node_stream_append(&stream, statement);
    }

    return stream;
}

Node* ast_parse_expression(AST* ast) {
    // Addition and subtraction is the binary operator that takes precedence.
    return ast_parse_addition_subtraction(ast);
}

Node* ast_parse_statement(AST* ast) {
    Node* statement = 0;

    // An asterisk at the start of a statement typically indicates a variable declaration, as
    // an asterisk is used for a pointer type.
    if (ast_next_is(ast, TOKEN_ASTERISK))
        statement = ast_parse_variable_declaration_statement(ast);

    else if (ast_next_is(ast, TOKEN_IDENTIFIER) && ast_after_next_is(ast, TOKEN_IDENTIFIER))
        statement = ast_parse_variable_declaration_statement(ast);

    else if (ast_next_is(ast, TOKEN_IDENTIFIER) && ast_after_next_is(ast, TOKEN_OPEN_PARENTHESIS))
        statement = ast_parse_function_call(ast);

    else if (ast_next_is_string(ast, TOKEN_KEYWORD, "return"))
        statement = ast_parse_return_statement(ast);

    else if (ast_next_is_string(ast, TOKEN_KEYWORD, "func") || ast_next_is_string(ast, TOKEN_KEYWORD, "extern"))
        statement = ast_parse_function_declaration_statement(ast);

    else {
        return ast_parse_expression(ast);
    }

    // If a statement was successfully parsed, expect a semicolon.
    if (statement && statement->node_type != NODE_FUNCTION_DECLARATION) {
        // All statements must end in a semicolon.
        ast_expect(ast, TOKEN_SEMICOLON);
    }

    return statement;
}

Node* ast_parse_addition_subtraction(AST* ast) {
    // Multiplication/division is less important than addition/subtraction.
    Node* left = ast_parse_multiplication_division(ast);
    if (!left) {
        return 0;
    }

    if (ast_next_is(ast, TOKEN_PLUS) || ast_next_is(ast, TOKEN_HYPHEN)) {
        Token operator_token = ast_consume(ast);

        // The next token is a plus or minus operator, parse a binary operation.
        Node* right = ast_parse_addition_subtraction(ast);
        if (!right) {
            return 0;
        }

        Operator operator_ = OPERATOR_PLUS;
        if (operator_token.type == TOKEN_HYPHEN) {
            operator_ = OPERATOR_MINUS;
        }

        return (Node*)binary_operation_node_create(operator_token.position, left, right, operator_);
    }

    return left;
}

Node* ast_parse_multiplication_division(AST* ast) {
    Node* left = ast_parse_value(ast);
    if (left == 0) {
        return 0;
    }

    if (ast_next_is(ast, TOKEN_ASTERISK) || ast_next_is(ast, TOKEN_SLASH)) {
        Token operator_token = ast_consume(ast);

        // The next token is a plus or minus operator, parse a binary operation.
        Node* right = ast_parse_expression(ast);
        if (right == 0) {
            return 0;
        }

        Operator operator_ = OPERATOR_MULTIPLY;
        if (operator_token.type == TOKEN_SLASH) {
            operator_ = OPERATOR_DIVIDE;
        }

        return (Node*)binary_operation_node_create(operator_token.position, left, right, operator_);
    }

    return left;
}

Node* ast_parse_value(AST* ast) {
    // If this value starts with an opening parenthesis, we need to parse
    // the expression within the parenthesis.
    if (ast_next_is(ast, TOKEN_OPEN_PARENTHESIS)) {
        ast_consume(ast);

        Node* expression = ast_parse_expression(ast);
        if (expression == 0) {
            return 0;
        }

        ast_expect(ast, TOKEN_CLOSE_PARENTHESIS);
        return expression;
    }

    if (ast_next_is_string(ast, TOKEN_KEYWORD, "true") || ast_next_is_string(ast, TOKEN_KEYWORD, "false"))
        return ast_parse_boolean_literal_expression(ast);

    Token next = ast_peek(ast);
    switch (next.type) {
    case TOKEN_NUMBER_LITERAL:
        return ast_parse_number_literal_expression(ast);

    case TOKEN_IDENTIFIER: {
        // If the following token is an opening parenthesis, this is a function call.
        if (ast_after_next_is(ast, TOKEN_OPEN_PARENTHESIS)) {
            return ast_parse_function_call(ast);
        }

        return ast_parse_identifier_reference_expression(ast);
    }

    case TOKEN_STRING_LITERAL:
        return ast_parse_string_literal_expression(ast);

    case TOKEN_INVALID:
        diagnostic_stream_push(&ast->diagnostics, next.position, true, "expected a value, but got end of file");
        return 0;

    default:
        diagnostic_stream_push(&ast->diagnostics, next.position, true, "unexpected token for value: '%s'",
                               token_to_string(next));
        return 0;
    }
}

// <number>
Node* ast_parse_number_literal_expression(AST* ast) {
    // The only token must be a number literal.
    Token token = ast_consume_type(ast, TOKEN_NUMBER_LITERAL);
    if (token.type == TOKEN_INVALID) {
        return 0;
    }

    // `token` comes from the `ast_expect` macro.
    return (Node*)number_literal_node_create(token.position, token.number);
}

// true | false
Node* ast_parse_boolean_literal_expression(AST* ast) {
    Token token = ast_consume_type(ast, TOKEN_KEYWORD);
    if (token.type == TOKEN_INVALID) {
        return 0;
    }

    return (Node*)boolean_literal_node_create(token.position, strcmp(token.string, "true"));
}

Node* ast_parse_string_literal_expression(AST* ast) {
    Token token = ast_consume_type(ast, TOKEN_STRING_LITERAL);
    if (token.type == TOKEN_INVALID) {
        return 0;
    }

    return (Node*)string_literal_node_create(token.position, token.string);
}

// <identifier>
Node* ast_parse_identifier_reference_expression(AST* ast) {
    // The first token must be an identifier.
    Token token = ast_consume_type(ast, TOKEN_IDENTIFIER);
    if (token.type == TOKEN_INVALID) {
        return 0;
    }

    return (Node*)identifier_reference_node_create(token.position, token.string);
}

Node* ast_parse_function_call(AST* ast) {
    // The first token must be an identifier.
    Token name_token = ast_consume_type(ast, TOKEN_IDENTIFIER);
    if (name_token.type == TOKEN_INVALID) {
        return 0;
    }

    // The next token must be an opening parenthesis.
    ast_expect(ast, TOKEN_OPEN_PARENTHESIS);

    NodeStream arguments;
    node_stream_initialize(&arguments, 1);

    // Attempt to parse the arguments to the function (if any).
    while (!ast_next_is(ast, TOKEN_CLOSE_PARENTHESIS)) {
        Node* value = ast_parse_expression(ast);
        if (!value) {
            return 0;
        }

        node_stream_append(&arguments, value);

        // If the next token is not a closing parenthesis, it must be a comma.
        if (!ast_next_is(ast, TOKEN_CLOSE_PARENTHESIS)) {
            ast_expect(ast, TOKEN_COMMA);
        }
    }

    // Must be closed by a closing parenthesis.
    ast_expect(ast, TOKEN_CLOSE_PARENTHESIS);

    return (Node*)function_call_node_create(name_token.position, name_token.string, arguments);
}

Type ast_parse_type(AST* ast) {
    // Variable declarations could be for a pointer type.
    bool is_pointer_type = false;

    // If the first token is an asterisk, this declaration is for a pointer type.
    if (ast_next_is(ast, TOKEN_ASTERISK)) {
        ast_consume(ast);
        is_pointer_type = true;
    }

    // The first token (or the one after the modifier) must be an identifier.
    Token type_token = ast_consume_type(ast, TOKEN_IDENTIFIER);
    if (type_token.type == TOKEN_INVALID) {
        return TYPE_INVALID;
    }

    // The type identifier token provided must be a valid type.
    TypeKind type_kind = type_kind_from_string(type_token.string);
    if (type_kind == TYPE_KIND_INVALID) {
        diagnostic_stream_push(&ast->diagnostics, type_token.position, true, "invalid type: '%s'", type_token.string);
        return TYPE_INVALID;
    }

    return type_create(type_kind, is_pointer_type);
}

// (*)<identifier> <identifier> = <value>
Node* ast_parse_variable_declaration_statement(AST* ast) {
    Type type = ast_parse_type(ast);
    if (type.kind == TYPE_KIND_INVALID) {
        return 0;
    }

    // The next token must be an identifier for the variable name.
    Token name_token = ast_consume_type(ast, TOKEN_IDENTIFIER);
    if (name_token.type == TOKEN_INVALID) {
        return 0;
    }

    ast_expect(ast, TOKEN_EQUALS);

    // The final token should be a valid value.
    Node* value = ast_parse_expression(ast);
    if (value == 0) {
        return 0;
    }

    LOG_DEBUG("ast", "parsed variable declaration node with value: '%s'", node_to_string(value));

    return (Node*)variable_declaration_node_create(name_token.position, name_token.string, type, value);
}

Parameter ast_parse_function_parameter(AST* ast) {
    // The first token must be the parameter's name.
    Token name_token = ast_consume_type(ast, TOKEN_IDENTIFIER);
    if (name_token.type == TOKEN_INVALID) {
        return PARAMETER_INVALID;
    }

    // The next token must be a colon.
    Token colon_token = ast_consume_type(ast, TOKEN_COLON);
    if (colon_token.type == TOKEN_INVALID) {
        return PARAMETER_INVALID;
    }

    // The last token must be a valid type identifier.
    Type type = ast_parse_type(ast);
    if (type.kind == TYPE_KIND_INVALID) {
        return PARAMETER_INVALID;
    }

    return parameter_create(name_token.string, type);
}

// (keyword: extern) <keyword: func> <identifier>(...) {...}
Node* ast_parse_function_declaration_statement(AST* ast) {
    // An extern function is parsed differently to a normal function.
    // The main difference being not having a function body surrounded by braces.
    bool is_extern_function = false;

    if (ast_next_is_string(ast, TOKEN_KEYWORD, "extern")) {
        ast_consume(ast);
        is_extern_function = true;
    }

    // The first token must be the "func" keyword.
    Token func_keyword_token = ast_consume_type_string(ast, TOKEN_KEYWORD, "func");
    if (func_keyword_token.type == TOKEN_INVALID) {
        return 0;
    }

    // The next token must be the token's name.
    Token name_token = ast_consume_type(ast, TOKEN_IDENTIFIER);
    if (name_token.type == TOKEN_INVALID) {
        return 0;
    }

    // The next token must be an opening parenthesis.
    ast_expect(ast, TOKEN_OPEN_PARENTHESIS);

    // Consume parameters until a close parenthesis is reached.
    Parameters parameters;
    parameters_initialize(&parameters, 1);

    while (!ast_next_is(ast, TOKEN_CLOSE_PARENTHESIS)) {
        LOG_DEBUG("ast", "attempting to parse parameter from '%s'", token_to_string(ast_peek(ast)));

        Parameter parameter = ast_parse_function_parameter(ast);
        if (parameter.type.kind == TYPE_KIND_INVALID) {
            return 0;
        }

        parameters_append(&parameters, parameter);

        // If the next token is not a closing parenthesis, it must be a comma.
        if (!ast_next_is(ast, TOKEN_CLOSE_PARENTHESIS)) {
            ast_expect(ast, TOKEN_COMMA);
        }
    }

    ast_expect(ast, TOKEN_CLOSE_PARENTHESIS);

    // The default return type is void.
    Type return_type = type_create(TYPE_KIND_VOID, false);

    // If a hyphen is after the closing parenthesis, we should parse a return type.
    if (ast_next_is(ast, TOKEN_HYPHEN)) {
        ast_consume(ast);

        // The next token must be a `>`.
        ast_expect(ast, TOKEN_RIGHT_ANGLE_BRACKET);

        // The final token must be a valid type.
        return_type = ast_parse_type(ast);
        if (return_type.kind == TYPE_KIND_INVALID) {
            return 0;
        }
    }

    // An external function does not have a body and must end in a semicolon.
    if (is_extern_function) {
        ast_expect(ast, TOKEN_SEMICOLON);
        return (Node*)function_declaration_node_create(func_keyword_token.position, name_token.string, parameters,
                                                       return_type, 0, true);
    }

    BlockNode* function_body = ast_parse_block(ast);
    if (!function_body) {
        return 0;
    }

    return (Node*)function_declaration_node_create(func_keyword_token.position, name_token.string, parameters,
                                                   return_type, function_body, false);
}

// <keyword: return> (value)
Node* ast_parse_return_statement(AST* ast) {
    // We already know that the first token is the `return` keyword.
    Token token = ast_consume_type(ast, TOKEN_KEYWORD);

    // If the next token is a semicolon, this return statement has no value.
    if (ast_next_is(ast, TOKEN_SEMICOLON)) {
        return (Node*)return_node_create(token.position, 0);
    }

    // The next token must be a valid expression.
    Node* value = ast_parse_expression(ast);
    if (!value) {
        return 0;
    }

    return (Node*)return_node_create(token.position, value);
}

BlockNode* ast_parse_block(AST* ast) {
    Token open_brace_token = ast_consume_type(ast, TOKEN_OPEN_BRACE);
    if (open_brace_token.type == TOKEN_INVALID) {
        return 0;
    }

    NodeStream body;
    node_stream_initialize(&body, 1);

    while (!ast_next_is(ast, TOKEN_CLOSE_BRACE)) {
        Node* statement = ast_parse_statement(ast);
        if (statement == 0) {
            return 0;
        }

        node_stream_append(&body, statement);
    }

    ast_expect(ast, TOKEN_CLOSE_BRACE);
    return block_node_create(open_brace_token.position, body);
}

void ast_destroy(AST* ast) {
    token_stream_destroy(&ast->token_stream);
    diagnostic_stream_destroy(&ast->diagnostics);
}
