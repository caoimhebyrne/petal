#include "ast.h"
#include "allocator.h"
#include "array.h"
#include "ast_statement.h"
#include "ast_type.h"
#include "diagnostic.h"
#include "lexer.h"
#include <assert.h>

IMPLEMENT_ARRAY_TYPE(StatementArray, statement_array, Statement*)
IMPLEMENT_ARRAY_TYPE(FunctionParameterArray, function_parameter_array, FunctionParameter)

void ast_parser_init(ASTParser* parser, Module* module, const TokenArray* tokens) {
    assert(parser != NULL && "NULL parser pointer passed to ast_parser_init");
    assert(module != NULL && "NULL module pointer passed to ast_parser_init");
    assert(tokens != NULL && "NULL tokens pointer passed to ast_parser_init");

    parser->tokens = tokens;
    parser->module = module;
    parser->cursor = 0;
}

/**
 * Returns whether the parser is at the end of the token stream.
 */
bool ast_parser_is_eof(const ASTParser* parser) {
    return parser->cursor >= parser->tokens->length;
}

/**
 * Returns the next available token in the token stream.
 */
const Token* ast_parser_consume(ASTParser* parser) {
    if (ast_parser_is_eof(parser)) {
        Diagnostic diagnostic = {0};

        diagnostic_init(
            &diagnostic,
            DIAGNOSTIC_KIND_ERROR,
            (Position){.module_id = parser->module->id},
            "expected any token, but reached the end of the file"
        );

        diagnostic_array_append(parser->module->diagnostics, diagnostic);
        return NULL;
    }

    return &parser->tokens->data[parser->cursor++];
}

/**
 * Returns the next available token in the token stream without advancing the cursor.
 */
const Token* ast_parser_peek(const ASTParser* parser) {
    if (ast_parser_is_eof(parser)) {
        return NULL;
    }

    return &parser->tokens->data[parser->cursor];
}

/**
 * Returns whether the token at the parser's current position is of a certain kind.
 */
bool ast_parser_peek_is(const ASTParser* parser, const TokenKind kind) {
    const Token* token = ast_parser_peek(parser);
    return token && token->kind == kind;
}

/**
 * Expects a certain token to be at the parser's current position, returning NULL if the token kind is not present.
 */
const Token* ast_parser_expect(ASTParser* parser, const TokenKind kind) {
    const Token* token = ast_parser_consume(parser);
    if (!token) {
        return NULL;
    }

    if (token->kind != kind) {
        Diagnostic diagnostic = {0};

        diagnostic_init_fmt(
            &diagnostic,
            parser->module->allocator,
            DIAGNOSTIC_KIND_ERROR,
            token->position,
            "expected token '%s' but got token '%s'",
            token_kind_to_string(kind),
            token_kind_to_string(token->kind)
        );

        diagnostic_array_append(parser->module->diagnostics, diagnostic);
        return NULL;
    }

    return token;
}

/**
 * Advances the cursor of the parser if the token at the current position is of a certain kind.
 */
bool ast_parser_consume_if(ASTParser* parser, const TokenKind kind) {
    if (!ast_parser_peek_is(parser, kind)) {
        return false;
    }

    return ast_parser_expect(parser, kind);
}

/**
 * Expects a certain keyword to be at the parser's current position, returning false if the keyword is not present.
 */
bool ast_parser_expect_keyword(ASTParser* parser, const Keyword keyword) {
    const Token* token = ast_parser_expect(parser, TOKEN_KIND_KEYWORD);
    if (!token) {
        return NULL;
    }

    if (token->keyword != keyword) {
        Diagnostic diagnostic = {0};

        diagnostic_init_fmt(
            &diagnostic,
            parser->module->allocator,
            DIAGNOSTIC_KIND_ERROR,
            token->position,
            "expected keyword: '%s' but got keyword '%s'"
        );

        diagnostic_array_append(parser->module->diagnostics, diagnostic);
        return NULL;
    }

    return token;
}

/**
 * Attempts to parse a single statement at the parser's current position.
 */
Statement* ast_parser_parse_statement(ASTParser* parser);

/**
 * Attempts to parse a function declaration statement at the parser's current position.
 */
Statement* ast_parser_parse_function_declaration(ASTParser* parser);

bool ast_parser_parse(ASTParser* parser, StatementArray* statements) {
    assert(parser != NULL && "NULL parser pointer passed to ast_parser_parse");
    assert(statements != NULL && "NULL statements pointer passed to ast_parser_parse");

    while (!ast_parser_is_eof(parser)) {
        Statement* statement = ast_parser_parse_statement(parser);
        if (!statement) {
            return false;
        }

        statement_array_append(statements, statement);
    }

    return true;
}

Statement* ast_parser_parse_statement(ASTParser* parser) {
    const Token* token = ast_parser_peek(parser);

    switch (token->kind) {
    case TOKEN_KIND_KEYWORD: {
        switch (token->keyword) {
        case KEYWORD_FUNC:
            return ast_parser_parse_function_declaration(parser);

        default: {
            Diagnostic diagnostic = {0};

            diagnostic_init_fmt(
                &diagnostic,
                parser->module->allocator,
                DIAGNOSTIC_KIND_ERROR,
                token->position,
                "expected to parse a statement, but got an unprocessable keyword: '%s'",
                keyword_to_string(token->keyword)
            );

            diagnostic_array_append(parser->module->diagnostics, diagnostic);
            return NULL;
        }
        }

        break;
    }

    default:
        break;
    }

    Diagnostic diagnostic = {0};

    diagnostic_init_fmt(
        &diagnostic,
        parser->module->allocator,
        DIAGNOSTIC_KIND_ERROR,
        token->position,
        "expected to parse a statement, but got an unprocessable token: '%s'",
        token_kind_to_string(token->kind)
    );

    diagnostic_array_append(parser->module->diagnostics, diagnostic);
    return NULL;
}

Statement* ast_parser_parse_function_declaration(ASTParser* parser) {
    if (!ast_parser_expect_keyword(parser, KEYWORD_FUNC)) {
        return NULL;
    }

    const Token* name_token = ast_parser_expect(parser, TOKEN_KIND_IDENTIFIER);
    if (!name_token) {
        return NULL;
    }

    if (!ast_parser_expect(parser, TOKEN_KIND_OPEN_PARENTHESIS)) {
        return NULL;
    }

    FunctionParameterArray parameters = {0};
    function_parameter_array_init(&parameters, parser->module->allocator);

    while (!ast_parser_peek_is(parser, TOKEN_KIND_CLOSE_PARENTHESIS)) {
        const Token* parameter_name_token = ast_parser_expect(parser, TOKEN_KIND_IDENTIFIER);
        if (!parameter_name_token) {
            return NULL;
        }

        if (!ast_parser_expect(parser, TOKEN_KIND_COLON)) {
            return NULL;
        }

        const Token* parameter_type_name_token = ast_parser_expect(parser, TOKEN_KIND_IDENTIFIER);
        if (!parameter_type_name_token) {
            return NULL;
        }

        const FunctionParameter parameter = {
            .name = parameter_name_token->string,
            .type = type_unknown(parameter_type_name_token->string)
        };

        function_parameter_array_append(&parameters, parameter);

        if (!ast_parser_peek_is(parser, TOKEN_KIND_CLOSE_PARENTHESIS) && !ast_parser_expect(parser, TOKEN_KIND_COMMA)) {
            return NULL;
        }
    }

    if (!ast_parser_expect(parser, TOKEN_KIND_CLOSE_PARENTHESIS)) {
        return NULL;
    }

    // By default, all functions return void unless the type is overridden.
    Type return_type = (Type){.kind = TYPE_KIND_VOID};

    if (ast_parser_consume_if(parser, TOKEN_KIND_HYPHEN)) {
        if (!ast_parser_expect(parser, TOKEN_KIND_RIGHT_ANGLE_BRACKET)) {
            return NULL;
        }

        const Token* return_type_name_token = ast_parser_expect(parser, TOKEN_KIND_IDENTIFIER);
        if (!return_type_name_token) {
            return NULL;
        }

        return_type = type_unknown(return_type_name_token->string);
    }

    if (!ast_parser_expect(parser, TOKEN_KIND_OPEN_BRACE)) {
        return NULL;
    }

    StatementArray statements = {0};
    statement_array_init(&statements, parser->module->allocator);

    while (!ast_parser_peek_is(parser, TOKEN_KIND_CLOSE_BRACE)) {
        Statement* statement = ast_parser_parse_statement(parser);
        if (!statement) {
            return NULL;
        }

        statement_array_append(&statements, statement);
    }

    if (!ast_parser_expect(parser, TOKEN_KIND_CLOSE_BRACE)) {
        return NULL;
    }

    // TODO: Factor out statement creation to another method.
    Statement* statement = allocator_alloc(parser->module->allocator, sizeof(Statement));
    if (!statement) {
        return NULL;
    }

    statement->kind = STATEMENT_KIND_FUNCTION_DECLARATION;
    statement->function_declaration = (FunctionDeclarationStatement){.body = statements,
                                                                     .name = name_token->string,
                                                                     .parameters = parameters,
                                                                     .return_type = return_type};

    return statement;
}
