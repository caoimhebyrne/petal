#include "core/module.h"
#include "ast/ast.h"
#include "ast/node.h"
#include "core/diagnostic.h"
#include "lexer/lexer.h"
#include "lexer/token.h"
#include "util/defer.h"
#include "util/file.h"
#include "util/string_builder.h"
#include "util/vector.h"
#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>

// Forward declarations:
void module_print_diagnostics(Module* module);

Module module_create(char* file_name) {
    return (Module){
        .diagnostics = vector_create(),
        .file_name = file_name,
    };
}

bool module_initialize(Module* module) {
    return vector_initialize(module->diagnostics, 1);
}

void module_compile(Module* module) {
    // To compile a module, we need to know its contents.
    module->file_contents = file_read(module->file_name);
    if (!module->file_contents.data) {
        return;
    }

    // The first stage of compilation is lexing, this produces a stream of tokens that can be parsed by the AST parser.
    auto lexer = lexer_create(&module->diagnostics, &module->file_contents);
    auto tokens = lexer_parse(&lexer);
    lexer_destroy(lexer);

    // If a non-allocated vector was returned, an error occurred.
    if (tokens.capacity == 0) {
        module_print_diagnostics(module);
        return;
    }

    // We have finished lexing the file, we can now take the tokens and construct an AST.
    auto ast = ast_create(&module->diagnostics, tokens);
    auto nodes = ast_parse(&ast);
    ast_destroy(ast);

    // If a non-allocated vector was returned, an error occurred.
    if (nodes.capacity == 0) {
        module_print_diagnostics(module);
        return;
    }

    for (size_t i = 0; i < nodes.length; i++) {
        auto node = vector_get(nodes, i);
        auto string defer(free_str) = node_to_string(node);

        if (string == nullptr) {
            printf("- !!! unable to stringify node: %d\n", node->kind);
        } else {
            printf("- %s\n", string);
        }
    }

    vector_destroy(nodes, node_destroy);
}

void module_print_diagnostics(Module* module) {
    if (module->diagnostics.length == 0) {
        return; // no diagnostics
    }

    Vector(char*) lines = vector_create();
    if (!vector_initialize(lines, 1)) {
        fprintf(stderr, "buy more ram if you want diagnostics???\n");
        return;
    }

    StringBuilder current_line = string_builder_create();
    if (string_builder_is_invalid(current_line)) {
        fprintf(stderr, "buy more ram if you want diagnostics???\n");
        return;
    }

    for (size_t i = 0; i < module->file_contents.length; i++) {
        if (string_builder_is_invalid(current_line)) {
            current_line = string_builder_create();
        }

        char character = module->file_contents.data[i];
        if (character == '\n') {
            vector_append(&lines, string_builder_finish(&current_line));
        } else {
            string_builder_append(&current_line, character);
        }
    }

    for (size_t i = 0; i < module->diagnostics.length; i++) {
        auto diagnostic = vector_get(module->diagnostics, i);

        printf(
            "%s: %s%s(%zu:%zu)%s: %s\n",
            ANSI_RED "error" ANSI_RESET,
            ANSI_LIGHT_GRAY,
            module->file_name,
            diagnostic.position.line + 1,
            diagnostic.position.column + 1,
            ANSI_RESET,
            diagnostic.message
        );

        if (diagnostic.position.line < lines.length) {
            auto line = vector_get(lines, diagnostic.position.line);

            const char* margin = ANSI_GRAY "|" ANSI_RESET;

            printf("   %s%3zu%s  %s  %s\n", ANSI_GRAY, diagnostic.position.line + 1, ANSI_RESET, margin, line);
            printf("        %s  ", margin);

            for (size_t i = 0; i < diagnostic.position.column; i++) {
                printf(" ");
            }

            for (size_t i = 0; i < diagnostic.position.length; i++) {
                printf(ANSI_YELLOW "^" ANSI_RESET);
            }

            printf("\n");
        }
    }

    vector_destroy(lines, free);
}

void module_destroy(Module module) {
    free(module.file_name);
    file_contents_destroy(module.file_contents);
    vector_destroy(module.diagnostics, diagnostic_destroy);

    // FIXME: Destroying a module should also destroy its dependencies.
}
