#ifndef __CODEGEN_H__
#define __CODEGEN_H__

#include "../ast/node.h"
#include "../ast/node/function_call.h"
#include "../ast/node/function_declaration.h"
#include "../ast/node/number_literal.h"
#include "../ast/node/return.h"
#include "../ast/node/variable_declaration.h"
#include "../string/string_builder.h"

typedef struct {
    // The node stream to read from.
    NodeStream node_stream;
} Codegen;

Codegen codegen_create(NodeStream node_stream);

char* codegen_generate(Codegen* codegen);

void codegen_generate_node(StringBuilder* builder, Node* node);
void codegen_generate_number_literal(StringBuilder* builder, NumberLiteralNode* node);
void codegen_generate_return(StringBuilder* builder, ReturnNode* node);
void codegen_generate_function_declaration(StringBuilder* builder, FunctionDeclarationNode* node);
void codegen_generate_variable_declaration(StringBuilder* builder, VariableDeclarationNode* node);
void codegen_generate_function_call(StringBuilder* builder, FunctionCallNode* node);

#endif
