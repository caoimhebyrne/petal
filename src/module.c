#include "module.h"
#include "array.h"
#include "file.h"
#include "lexer.h"
#include "logger.h"
#include <string.h>

bool module_init(Module* module, Allocator* allocator, const char* file_path) {
    StringBuffer file_path_buffer = {0};
    string_buffer_init(&file_path_buffer, allocator);
    string_buffer_append_many(&file_path_buffer, file_path, strlen(file_path));

    StringBuffer source_buffer = {0};
    string_buffer_init(&source_buffer, allocator);

    if (!file_read(file_path, &source_buffer)) {
        return false;
    }

    module->allocator = allocator;
    module->file_path = file_path_buffer;
    module->source = source_buffer;

    return true;
}

bool module_parse(Module* module) {
    Lexer lexer = {0};
    lexer_init(&lexer, module->allocator, &module->source);

    TokenArray tokens = {0};
    token_array_init(&tokens, module->allocator);

    if (!lexer_parse(&lexer, &tokens)) {
        return false;
    }

    for (size_t i = 0; i < tokens.length; i++) {
        const Token token = tokens.data[i];

        switch (token.kind) {
        case TOKEN_KIND_EQUALS:
            log_info("equals");
            break;

        case TOKEN_KIND_IDENTIFIER:
            log_info("identifier '%s'", token.string);
            break;

        case TOKEN_KIND_NUMBER:
            log_info("number %f", token.number);
            break;

        case TOKEN_KIND_CLOSE_BRACE:
            log_info("close brace");
            break;

        case TOKEN_KIND_OPEN_BRACE:
            log_info("open brace");
            break;

        case TOKEN_KIND_OPEN_PARENTHESIS:
            log_info("open parenthesis");
            break;

        case TOKEN_KIND_CLOSE_PARENTHESIS:
            log_info("close parenthesis");
            break;

        case TOKEN_KIND_SEMICOLON:
            log_info("semicolon");
            break;

        case TOKEN_KIND_COMMA:
            log_info("comma");
            break;
        }
    }

    return true;
}
