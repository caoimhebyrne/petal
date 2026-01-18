#include "array.h"

typedef struct Expression Expression;

DEFINE_ARRAY_TYPE(ExpressionArray, expression_array, Expression*)

/**
 * Represents the different kinds of expressions that are available.
 */
typedef enum {
    /**
     * A number literal.
     */
    EXPRESSION_KIND_NUMBER_LITERAL,

    /**
     * A function call.
     */
    EXPRESSION_KIND_FUNCTION_CALL,

    /**
     * An identifier reference/
     */
    EXPRESSION_KIND_IDENTIFIER_REFERENCE,
} ExpressionKind;

/**
 * A function call.
 */
typedef struct {
    /**
     * The name of the function being called.
     */
    StringBuffer name;

    /**
     * The arguments being passed to the function.
     */
    ExpressionArray arguments;
} FunctionCall;

/**
 * The AST of Petal is made up of statements and expressions.
 *
 * An expression is typically a node that produces a value, for example: a number literal.
 */
struct Expression {
    /**
     * The kind of expression that this is.
     */
    ExpressionKind kind;

    union {
        /**
         * EXPRESSION_KIND_NUMBER_LITERAL.
         */
        float number_literal;

        /**
         * EXPRESSION_KIND_FUNCTION_CALL
         */
        FunctionCall function_call;

        /**
         * EXPRESSION_KIND_IDENTIFIER_REFERENCE
         */
        StringBuffer identifier;
    };
};
