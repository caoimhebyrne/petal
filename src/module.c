#include "module.h"
#include "allocator.h"
#include "array.h"
#include "ast.h"
#include "ast_statement.h"
#include "diagnostic.h"
#include "file.h"
#include "lexer.h"
#include "logger.h"
#include <string.h>

static size_t global_module_id = 1;

bool module_init(Module* module, Allocator* allocator, DiagnosticArray* diagnostics, const char* file_path) {
    StringBuffer source_buffer = {0};
    string_buffer_init(&source_buffer, allocator);

    if (!file_read(file_path, &source_buffer)) {
        return false;
    }

    StringBuffer file_path_buffer = {0};
    string_buffer_init_from_cstr(&file_path_buffer, allocator, file_path);

    StringBuffer name_buffer = {0};
    string_buffer_init_from_cstr(&name_buffer, allocator, file_path);

    // The module name is the name of the file without any parent directory or file extension.
    string_buffer_trim_from(&name_buffer, PATH_SEPARATOR);
    string_buffer_trim_until(&name_buffer, '.');

    module->id = (ModuleId){.unwrap = global_module_id++};
    module->allocator = allocator;
    module->diagnostics = diagnostics;
    module->file_path = file_path_buffer;
    module->name = name_buffer;
    module->source = source_buffer;

    log_info(
        "initialized module '%.*s' (%zu) from path '%s'",
        (int)name_buffer.length,
        name_buffer.data,
        module->id.unwrap,
        file_path
    );

    return true;
}

bool module_parse(Module* module, StatementArray* statements) {
    Lexer lexer = {0};
    lexer_init(&lexer, module);

    TokenArray tokens = {0};
    token_array_init(&tokens, module->allocator);

    if (!lexer_parse(&lexer, &tokens)) {
        return false;
    }

    ASTParser parser = {0};
    ast_parser_init(&parser, module, &tokens);

    if (!ast_parser_parse(&parser, statements)) {
        return false;
    }

    return true;
}
