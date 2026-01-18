#include "ast.h"
#include "array.h"
#include "ast_statement.h"
#include <assert.h>

IMPLEMENT_ARRAY_TYPE(StatementArray, statement_array, Statement*)

void ast_parser_init(ASTParser* parser, Module* module, const TokenArray* tokens) {
    assert(parser != NULL && "NULL parser pointer passed to ast_parser_init");
    assert(module != NULL && "NULL module pointer passed to ast_parser_init");
    assert(tokens != NULL && "NULL tokens pointer passed to ast_parser_init");

    parser->tokens = tokens;
    parser->module = module;
    parser->cursor = 0;
}

bool ast_parser_parse(ASTParser* parser, StatementArray* statements) {
    assert(parser != NULL && "NULL parser pointer passed to ast_parser_parse");
    assert(statements != NULL && "NULL statements pointer passed to ast_parser_parse");

    return false;
}
