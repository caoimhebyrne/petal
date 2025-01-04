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

void module_compile(Module* module) {
    // To compile a module, we need to know its contents.
    auto file_contents = file_read(module->file_name);
    if (!file_contents.data) {
        return;
    }

    // The first stage of compilation is lexing, this produces a stream of tokens that can be parsed by the AST parser.
    auto lexer = lexer_create(file_contents);
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
    for (size_t i = 0; i < module->diagnostics.length; i++) {
        Diagnostic diagnostic = vector_get(module->diagnostics, i);
        printf(
            "%s: %s%s(%zu:%zu)%s: %s\n",
            ANSI_RED "error" ANSI_RESET,
            ANSI_LIGHT_GRAY,
            module->file_name,
            diagnostic.position.line + 1,
            diagnostic.position.column + 2, // 1 for zero indexing, and 1 for "end of file".
            ANSI_RESET,
            diagnostic.message
        );
    }
}

void module_destroy(Module module) {
    vector_destroy(module.diagnostics, diagnostic_destroy);

    free(module.file_name);

    // FIXME: Destroying a module should also destroy its dependencies.
}
