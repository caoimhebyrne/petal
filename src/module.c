#include "module.h"
#include "ast/ast.h"
#include "ast/node.h"
#include "ast/node/function_declaration.h"
#include "ast/node/import.h"
#include "codegen/llvm_codegen.h"
#include "lexer/lexer.h"
#include "logger.h"
#include "stream.h"
#include "string/format_string.h"
#include "typechecker/typechecker.h"
#include <libgen.h>
#include <llvm-c/Core.h>
#include <llvm-c/Linker.h>
#include <stdbool.h>
#include <string.h>

CREATE_STREAM(ModuleDependencies, module_dependencies, Module);

// Forward declarations:
bool module_resolve_dependencies(Module* module);
bool module_link_with_dependencies(Module* module);

Module module_create(char* file_name) {
    ModuleDependencies dependencies;
    module_dependencies_initialize(&dependencies, 1);

    LOG_DEBUG("module", "created module: '%s'", file_name);

    return (Module){
        .file_name = file_name,
        .llvm_module = 0,
        .llvm_context = LLVMContextCreate(),
        .node_stream = (NodeStream){},
        .dependencies = dependencies,
    };
}

Module module_create_dependency(Module* parent, char* file_name) {
    ModuleDependencies dependencies;
    module_dependencies_initialize(&dependencies, 1);

    LOG_DEBUG("module", "created dependency module: '%s'", file_name);

    return (Module){
        .file_name = file_name,
        .llvm_module = 0,
        .llvm_context = parent->llvm_context,
        .node_stream = (NodeStream){},
        .dependencies = dependencies,
    };
}

bool module_compile(Module* module) {
    Lexer lexer;
    if (!lexer_initialize(&lexer, module->file_name)) {
        return false;
    }

    TokenStream token_stream = lexer_parse(&lexer);
    if (lexer.diagnostics.length != 0) {
        diagnostic_stream_print(&lexer.diagnostics, module->file_name);

        token_stream_destroy(&token_stream);
        lexer_destroy(&lexer);

        return false;
    }

    AST ast;
    if (!ast_initialize(&ast, token_stream)) {
        return false;
    }

    module->node_stream = ast_parse(&ast);
    if (ast.diagnostics.length != 0) {
        diagnostic_stream_print(&ast.diagnostics, module->file_name);
        return false;
    }

    // Attempt to resolve dependencies.
    if (!module_resolve_dependencies(module)) {
        return false;
    }

    Typechecker typechecker = typechecker_create();
    typechecker_run(&typechecker, &module->node_stream);

    if (typechecker.diagnostics.length != 0) {
        diagnostic_stream_print(&typechecker.diagnostics, module->file_name);
        typechecker_destroy(&typechecker);

        return 0;
    }

    typechecker_destroy(&typechecker);

    LLVMCodegen codegen = llvm_codegen_create(module->llvm_context, module->file_name, module->node_stream);
    module->llvm_module = codegen.module;

    llvm_codegen_generate(&codegen);
    llvm_codegen_destroy(&codegen);

    if (codegen.diagnostics.length != 0) {
        diagnostic_stream_print(&codegen.diagnostics, module->file_name);
        llvm_codegen_destroy(&codegen);

        return 0;
    }

    if (!module_link_with_dependencies(module)) {
        return false;
    }

    LOG_DEBUG("module", "compiled module: '%s'", module->file_name);
    return true;
}

bool module_resolve_dependencies(Module* module) {
    // We only need to resolve depenedncies if there are import statements.
    NodeStream node_stream;
    node_stream_initialize(&node_stream, module->node_stream.length);

    // The working directory to resolve other modules from can be derived from the filename.
    char* working_directory = dirname(strdup(module->file_name));

    // Before we add the existing nodes, we must create external functions for any import statements.
    for (size_t i = 0; i < module->node_stream.length; i++) {
        Node* node = module->node_stream.data[i];
        if (node->node_type != NODE_IMPORT) {
            continue;
        }

        ImportNode* import_node = (ImportNode*)node;

        // The imported module can be resolved relative to the current working directory.
        char* dependency_path = format_string("%s/%s.petal", working_directory, import_node->module_name);

        // We must compile the module before we can use it as a dependency.
        Module dependency = module_create_dependency(module, dependency_path);
        if (!module_compile(&dependency)) {
            return false;
        }

        LOG_DEBUG("module", "compiled dependency module: '%s'", dependency.file_name);

        // The module has been compiled, we can generate extern function declarations for its functions.
        for (size_t j = 0; j < dependency.node_stream.length; j++) {
            Node* dependency_node = dependency.node_stream.data[j];
            if (dependency_node->node_type != NODE_FUNCTION_DECLARATION) {
                continue;
            }

            FunctionDeclarationNode* original_declaration = (FunctionDeclarationNode*)dependency_node;
            FunctionDeclarationNode* extern_declaration = function_declaration_node_create(
                original_declaration->position,
                original_declaration->name,
                original_declaration->parameters,
                original_declaration->return_type,
                0,
                true
            );

            LOG_DEBUG("module", "generated declaration for imported function: '%s'", original_declaration->name);
            node_stream_append(&node_stream, (Node*)extern_declaration);
        }

        module_dependencies_append(&module->dependencies, dependency);
    }

    // We can now add the existing nodes.
    for (size_t i = 0; i < module->node_stream.length; i++) {
        node_stream_append(&node_stream, module->node_stream.data[i]);
    }

    // This is the module's new node stream.
    module->node_stream = node_stream;

    return true;
}

bool module_link_with_dependencies(Module* module) {
    for (size_t i = 0; i < module->dependencies.length; i++) {
        Module dependency = module->dependencies.data[i];
        LOG_DEBUG("module", "linking '%s' to '%s'", dependency.file_name, module->file_name);

        if (LLVMLinkModules2(module->llvm_module, dependency.llvm_module)) {
            LOG_ERROR("module", "failed to link '%s' to '%s'", dependency.file_name, module->file_name);
            return false;
        }
    }

    return true;
}
