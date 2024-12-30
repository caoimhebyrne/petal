#include "binary_operation.h"
#include "../../string/format_string.h"
#include <stdlib.h>

BinaryOperationNode* binary_operation_node_create(Position position, Node* left, Node* right, Operator operator_) {
    BinaryOperationNode* node = malloc(sizeof(BinaryOperationNode));
    if (!node) {
        return 0;
    }

    node->node_type = NODE_BINARY_OPERATION;
    node->position = position;
    node->left = left;
    node->right = right;
    node->operator_ = operator_;
    node->type = 0;

    return node;
}

char* binary_operation_node_to_string(BinaryOperationNode* node) {
    return format_string(
        "binary operation (operator: '%s', left: '%s', right: '%s')",
        operator_to_string(node->operator_),
        node_to_string(node->left),
        node_to_string(node->right)
    );
}

char* operator_to_string(Operator operator_) {
    switch (operator_) {
    case OPERATOR_PLUS:
        return "plus";

    case OPERATOR_MINUS:
        return "minus";

    case OPERATOR_DIVIDE:
        return "divide";

    case OPERATOR_MULTIPLY:
        return "multiply";
    }

    return "unknown operator";
}
