#include "module.h"
#include "allocator.h"
#include "array.h"
#include "ast.h"
#include "diagnostic.h"
#include "file.h"
#include "lexer.h"
#include "logger.h"
#include <string.h>

static size_t global_module_id = 1;

void print_node_tree(Allocator* allocator, const NodeArray* nodes, size_t depth);

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

bool module_parse(Module* module) {
    Lexer lexer = {0};
    lexer_init(&lexer, module);

    TokenArray tokens = {0};
    token_array_init(&tokens, module->allocator);

    if (!lexer_parse(&lexer, &tokens)) {
        return false;
    }

    ASTParser ast_parser = {0};
    ast_parser_init(&ast_parser, module->allocator, module->diagnostics, module->id, &tokens);

    NodeArray nodes = {0};
    node_array_init(&nodes, module->allocator);

    if (!ast_parser_parse(&ast_parser, &nodes)) {
        return false;
    }

    print_node_tree(module->allocator, &nodes, 0);

    return true;
}

void print_node_tree(Allocator* allocator, const NodeArray* nodes, size_t depth) {
    StringBuffer padding = {0};
    string_buffer_init(&padding, allocator);

    for (size_t i = 0; i < depth * 4; i++) {
        string_buffer_append(&padding, ' ');
    }

    for (size_t i = 0; i < nodes->length; i++) {
        const Node* node = nodes->data[i];

        switch (node->kind) {
        case NODE_KIND_FUNCTION_DECLARATION: {
            const FunctionDeclarationNode function_declaration = node->function_declaration;

            log_info(
                "%.*sfunction declaration '%.*s'",
                (int)padding.length,
                padding.data,
                (int)function_declaration.name.length,
                function_declaration.name.data
            );

            print_node_tree(allocator, &function_declaration.body, depth + 1);

            break;
        }

        case NODE_KIND_RETURN: {
            const ReturnNode return_ = node->return_;

            log_info("%.*sreturn %p", (int)padding.length, padding.data, return_.value);

            break;
        }
        }
    }
}
