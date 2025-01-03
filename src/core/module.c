#include "core/module.h"
#include "ast/ast.h"
#include "ast/node.h"
#include "lexer/lexer.h"
#include "lexer/token.h"
#include "util/file.h"
#include "util/vector.h"
#include <stdlib.h>
#include <sys/stat.h>

Module module_create(char* file_name) {
    return (Module){file_name};
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
        return;
    }

    // We have finished lexing the file, we can now take the tokens and construct an AST.
    auto ast = ast_create(tokens);
    auto nodes = ast_parse(&ast);
    ast_destroy(ast);

    // If a non-allocated vector was returned, an error occurred.
    if (nodes.capacity == 0) {
        return;
    }

    vector_destroy(nodes, node_destroy);
}

void module_destroy(Module module) {
    // FIXME: Destroying a module should also destroy its dependencies.
    free(module.file_name);
}
