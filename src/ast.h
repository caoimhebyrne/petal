#pragma once

#include "ast_statement.h"
#include "lexer.h"
#include "module.h"

/**
 * Parses tokens into an AST.
 */
typedef struct {
    /**
     * The module that the tokens belong to.
     */
    Module* module;

    /**
     * The tokens to parse.
     */
    const TokenArray* tokens;

    /**
     * The position that the parser is at in the TokenArray.
     */
    size_t cursor;
} ASTParser;

/**
 * Initializes an [ASTParser] with the provided module and tokens.
 */
void ast_parser_init(ASTParser* parser, Module* module, const TokenArray* tokens);

/**
 * Attempts to parse the tokens within this parser, appending statements onto the provided [StatementArray].
 */
bool ast_parser_parse(ASTParser* parser, StatementArray* statements);
