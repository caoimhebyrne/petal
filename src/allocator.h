#pragma once

#include <stddef.h>

// A region of memory allocated by an Allocator.
typedef struct AllocatorRegion AllocatorRegion;

struct AllocatorRegion {
    // The start of the memory owned by this region.
    char* start;

    // The address of the next free part of memory.
    char* cursor;

    // The capacity of the region. Once a region has been allocated, it cannot be resized.
    size_t capacity;

    // The pointer to the next region of memory, this may be null.
    AllocatorRegion* next;
};

// The main allocator used by Petal during compilation and execution.
typedef struct {
    // The first region owned by this allocator, may be null.
    AllocatorRegion* first;
} Allocator;

// Initializes an Allocator with a single default region.
// If the region could not be created, this function will abort the program.
void allocator_init(Allocator* allocator);

// Allocates memory of a certain [size] on the next available region.
// If a region does not exist with [size] bytes available, then a new region will be allocated with enough capacity fo
// the data.
// This function will return NULL if the allocation did not succeed.
void* allocator_alloc(Allocator* allocator, const size_t size);

// De-allocates all memory controlled by the provided allocator.
void allocator_clean(Allocator* allocator);
