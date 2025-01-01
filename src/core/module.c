#include "core/module.h"
#include "lexer/lexer.h"
#include "lexer/token.h"
#include "util/file.h"
#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>

Module module_create(char* file_name) {
    return (Module){file_name};
}

void module_compile(Module* module) {
    // To compile a module, we need to know its contents.
    FileContents file_contents = file_read(module->file_name);
    if (!file_contents.data) {
        return;
    }

    // The first stage of compilation is lexing, this produces a stream of tokens that can be parsed by the AST parser.
    Lexer lexer = lexer_create(file_contents);
    TokenVector tokens = lexer_parse(&lexer);
    lexer_destroy(lexer);

    // If no vector was returned from lexer_parse, an error occurred during parsing.
    if (tokens.capacity == 0) {
        return;
    }

    // We have finished lexing the file, we can now take the tokens and construct an AST.
    printf("module: parsed %zu token(s)\n", tokens.length);
    vector_destroy(tokens, token_destroy);
}

void module_destroy(Module module) {
    // FIXME: Destroying a module should also destroy its dependencies.
    free(module.file_name);
}
