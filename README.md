# petal

Petal is a hobby programming language. The goal is for the language to compile into a binary, but at the moment it is
interpreted.

## Building Petal

A [Makefile](./Makefile) is provided to build Petal. Any C99 compiler should be able to compile Petal.

To build Petal, run the `build` target:

```
$ make build
```

This will create a `petal` binary in `./build`.

## Using Petal

After you have [compiled Petal](#building-petal), feel free to run any of the examples in the [examples](./examples)
directory:

```
$ ./build/petal ./examples/00_init.petal
info: initialized module '00_init' (1) from path './examples/00_init.petal'
info: vm executed successfully, exit code: 123
```

## Advanced Information

This section contains some advanced information for those wishing to contribute to the project, or those that wish to
experiment with the internals of Petal.

### Behavior Flags

When compiling Petal, you can define any of these flags (add `-D{flag_name}` to the `CFLAGS` variable) to change
the behavior of the binary:

1. `PETAL_ALLOCATOR_DEBUG`: Enables debug logging for the arena allocator. Example:

   ```
    debug: allocator creating a new region of size 512
    info: initialized module '00_init' (1) from path './examples/00_init.petal'
    debug: allocator creating a new region of size 512
    debug: allocator creating a new region of size 576
    debug: allocator creating a new region of size 1152
    info: vm executed successfully, exit code: 123
    debug: allocator freeing region at 0x2ccac720 (capacity = 512)
    debug: allocator freeing region at 0x2ccacb30 (capacity = 512)
    debug: allocator freeing region at 0x2ccacd60 (capacity = 576)
    debug: allocator freeing region at 0x2ccacfd0 (capacity = 1152)
   ```

## License

This project is licensed under the [MIT license](./LICENSE).
