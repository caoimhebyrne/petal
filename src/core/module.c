#include "core/module.h"
#include "ast/ast.h"
#include "ast/node.h"
#include "core/diagnostic.h"
#include "lexer/lexer.h"
#include "lexer/token.h"
#include "util/defer.h"
#include "util/file.h"
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

bool module_compile(Module* module) {
    // To compile a module, we need to know its contents.
    module->file_contents = file_read(module->file_name);
    if (!module->file_contents.data) {
        return false;
    }

    // The first stage of compilation is lexing, this produces a stream of tokens that can be parsed by the AST parser.
    auto lexer = lexer_create(&module->diagnostics, &module->file_contents);
    auto tokens = lexer_parse(&lexer);
    lexer_destroy(lexer);

    // If a non-allocated vector was returned, an error occurred.
    if (tokens.capacity == 0) {
        module_print_diagnostics(module);
        return false;
    }

    // We have finished lexing the file, we can now take the tokens and construct an AST.
    auto ast = ast_create(&module->diagnostics, tokens);
    auto nodes = ast_parse(&ast);
    ast_destroy(ast);

    // If a non-allocated vector was returned, an error occurred.
    if (nodes.capacity == 0) {
        module_print_diagnostics(module);

        vector_destroy(nodes, node_destroy);
        return false;
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
    return true;
}

void module_print_diagnostics(Module* module) {
    if (module->diagnostics.length == 0) {
        return; // no diagnostics
    }

    // Parse the module's source into lines.
    // This will be used to print lines when printing diagnostics.
    auto source_lines = file_contents_lines(module->file_contents);

    for (size_t i = 0; i < module->diagnostics.length; i++) {
        auto diagnostic = vector_get(&module->diagnostics, i);

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

        // Ensure the line index is within the vector bounds.
        if (diagnostic.position.line < source_lines.length) {
            auto line = vector_get(&source_lines, diagnostic.position.line);
            auto margin = ANSI_GRAY "|" ANSI_RESET;

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

    vector_destroy(source_lines, free);
}

void module_destroy(Module* module) {
    free(module->file_name);
    file_contents_destroy(module->file_contents);
    vector_destroy(module->diagnostics, diagnostic_destroy);

    // FIXME: Destroying a module should also destroy its dependencies.
}
