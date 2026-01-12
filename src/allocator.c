#include "allocator.h"
#include "logger.h"
#include "math.h"
#include <assert.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define ALLOCATOR_REGION_DEFAULT_CAPACITY_BYTES 512

// Creates a new AllocatorRegion with a certain [minimum_size].
// The size of the region will be at least [ALLOCATOR_REGION_DEFAULT_CAPACITY_BYTES], and at most [minimum_size] bytes.
AllocatorRegion* allocator_region_create(const size_t minimum_size) {
    const size_t region_size = min(minimum_size, ALLOCATOR_REGION_DEFAULT_CAPACITY_BYTES);

    log_debug("allocator creating a new region of size %zu", region_size);

    AllocatorRegion* region = malloc(sizeof(AllocatorRegion) + region_size);
    assert(region != NULL && "Failed to create new allocator region");

    region->start = (char*)region + sizeof(AllocatorRegion);
    region->cursor = region->start;
    region->capacity = region_size;
    region->next = NULL;

    return region;
}

// Returns the number of bytes remaining in the provided [AllocatorRegion].
size_t allocator_region_get_remaining_bytes(const AllocatorRegion* region) {
    assert(region->start != NULL && "Region start was NULL?");
    assert(region->cursor != NULL && "Region cursor was NULL?");

    const ptrdiff_t used_bytes = region->cursor - region->start;
    return region->capacity - used_bytes;
}

// Allocates memory of a certain [size] on a certain region.
// This function will return NULL if the allocation did not succeed.
void* allocator_region_alloc(AllocatorRegion* region, const size_t size) {
    assert(region->cursor != NULL && "Region cursor was NULL?");
    assert(region->capacity >= allocator_region_get_remaining_bytes(region) && "Region does not have enough space");

    char* cursor = region->cursor;
    region->cursor = cursor + size;

    return (void*)cursor;
}

// Adds a new region to this allocator with a certain [minimum_size].
AllocatorRegion* allocator_create_region(Allocator* allocator, const size_t minimum_size) {
    // We need to find the last region to allocate a new one.
    AllocatorRegion* last_region = allocator->first;
    while (last_region->next != NULL) {
        last_region = last_region->next;
    }

    assert(last_region != NULL && "Allocator must be initialized before calling create_region!");

    AllocatorRegion* new_region = allocator_region_create(minimum_size);
    last_region->next = new_region;

    return new_region;
}

void allocator_init(Allocator* allocator) {
    allocator->first = allocator_region_create(0);
}

void* allocator_alloc(Allocator* allocator, const size_t size) {
    assert(allocator->first != NULL && "Allocator must be initialized before calling alloc!");

    // If a region exists that has enough capacity remaining, a pointer to that region's cursor should be returned.
    AllocatorRegion* region = allocator->first;
    while (region != NULL) {
        if (allocator_region_get_remaining_bytes(region) >= size) {
            return allocator_region_alloc(region, size);
        }

        region = region->next;
    }

    // A region could not be found that has enough space, we must allocate a new region that can take the data.
    AllocatorRegion* new_region = allocator_create_region(allocator, size);
    return allocator_region_alloc(new_region, size);
}

void* allocator_realloc(Allocator* allocator, const void* data, const size_t old_size, const size_t new_size) {
    void* new_data = allocator_alloc(allocator, new_size);
    if (!new_data) {
        return NULL;
    }

    memcpy(new_data, data, old_size);

    // TODO: Move to allocator_free.

    // We should attempt to free the data in the old region to allow it to be re-used.
    // If the data is at its region's current cursor, then we can easily reclaim the memory by reversing the cursor
    // by [old_size] bytes.
    AllocatorRegion* region = allocator->first;
    while (region != NULL) {
        assert(region->cursor != NULL && "Region had a null cursor!");

        // If the pointer was allocated within this region, then we no longer need to search.
        const void* region_end = region->start + region->capacity;
        if (region->start <= (char*)data && region_end >= data) {
            // If the region's last allocated pointer was this pointer, then we can reset it.
            const void* potential_last_allocated_pointer = region->cursor - old_size;
            if (potential_last_allocated_pointer == data) {
                region->cursor -= old_size;
            }

            break;
        }

        region = region->next;
    }

    return new_data;
}

void allocator_clean(Allocator* allocator) {
    assert(allocator->first != NULL && "Allocator must be initialized before calling clean!");

    AllocatorRegion const* region = allocator->first;
    while (region != NULL) {
        assert(region->start != NULL && "Region start was NULL?");
        // We must store the next region before we do anything else as we will be free'ing the current one soon.
        const AllocatorRegion* next_region = region->next;

        void* region_address = region->start - sizeof(AllocatorRegion);
        log_debug("allocator freeing region at %p (capacity = %zu)", region_address, region->capacity);

        // Any subsequently allocated regions have a header allocated just before them.
        free(region_address);

        region = next_region;
    }
}
