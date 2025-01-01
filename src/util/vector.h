#ifndef __UTIL_VECTOR_H__
#define __UTIL_VECTOR_H__

#include <stdbool.h>
#include <stddef.h>

typedef struct {
    // The array of items within the vector.
    void** items;

    // The length of the `items` array.
    size_t size;

    // The amount of memory allocated for `items`.
    size_t capacity;
} Vector;

// Initializes a new Vector with an initial capacity.
// Returns:
// - A pointer to a Vector if successful, otherwise null.
Vector* vector_create(size_t capacity);

// Appends an item to a vector, which may involve resizing the vector.
// This makes a copy of the item.
// Parameters:
// - vector: The vector to append to.
// - item: The item to append.
// Returns whether the operation was successful.
bool vector_append(Vector* vector, void* item, size_t item_size);

// Destroys a Vector.
// If you have other data to destroy within your items (e.g. each item has a string allocated within it), you must
// iterate over all of the items, and destroy them individually.
// Parameters:
// - vector: The vector to destroy.
void vector_destroy(Vector* vector);

#endif // __UTIL_VECTOR_H__
