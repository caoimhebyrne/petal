#include "ast.h"
#include "array.h"
#include "ast_statement.h"

IMPLEMENT_ARRAY_TYPE(StatementArray, statement_array, Statement*)

void ast_parser_init(ASTParser* parser, Module* module, const TokenArray* tokens) {
    parser->tokens = tokens;
    parser->module = module;
    parser->cursor = 0;
}

bool ast_parser_parse(ASTParser* parser, StatementArray* statements) {
    (void)parser;
    (void)statements;
    return false;
}
