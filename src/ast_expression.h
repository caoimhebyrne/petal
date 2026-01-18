/**
 * Represents the different kinds of expressions that are available.
 */
typedef enum {
    /**
     * A number literal.
     */
    EXPRESSION_KIND_NUMBER_LITERAL,
} ExpressionKind;

/**
 * The AST of Petal is made up of statements and expressions.
 *
 * An expression is typically a node that produces a value, for example: a number literal.
 */
typedef struct {
    /**
     * The kind of expression that this is.
     */
    ExpressionKind kind;

    union {
        /**
         * EXPRESSION_KIND_NUMBER_LITERAL.
         */
        float number_literal;
    };
} Expression;
