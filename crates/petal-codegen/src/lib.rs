use cranelift::{
    module::{self, Module},
    native, object,
    prelude::*,
};
use petal_ast::Definition;
use petal_typecheck::TypecheckerResult;

use crate::function::FunctionCodegen;

mod expression;
mod function;
mod statement;

struct Codegen {
    builder_context: FunctionBuilderContext,
    context: codegen::Context,
    module: object::ObjectModule,
}

impl Codegen {
    /// Create a new Cranelift [`Codegen`].
    ///
    /// The code for this codegen has been based off the JIT demo, and has been adjusted to emit an object instead of
    /// a JIT module.
    ///
    /// https://github.com/bytecodealliance/cranelift-jit-demo/blob/main/src/jit.rs
    fn new<V>(object_name: V) -> Self
    where
        V: Into<Vec<u8>>,
    {
        let flags_builder = settings::builder();

        let isa_builder = native::builder().expect("host machine is not supported by cranelift");

        let isa = isa_builder
            .finish(settings::Flags::new(flags_builder))
            .expect("failed to build isa");

        let object_builder = object::ObjectBuilder::new(isa, object_name, module::default_libcall_names())
            .expect("failed to create objectbuilder");

        let module = object::ObjectModule::new(object_builder);

        Self {
            builder_context: FunctionBuilderContext::new(),
            context: module.make_context(),
            module,
        }
    }

    /// Finishes compilation by returning the bytes of the final object file.
    fn finish(self) -> Vec<u8> {
        let product = self.module.finish();
        product.emit().expect("failed to emit object")
    }
}

impl Codegen {
    /// Visits a top level definition in a program.
    fn visit_definition(&mut self, definition: &Definition) {
        match definition {
            Definition::Function(function) => {
                let function_codegen = FunctionCodegen {
                    builder: FunctionBuilder::new(&mut self.context.func, &mut self.builder_context),
                    module: &mut self.module,
                };

                function_codegen.build(&function.body);

                let id = self
                    .module
                    .declare_function(&function.name, module::Linkage::Export, &self.context.func.signature)
                    .expect("failed to declare function");

                self.module
                    .define_function(id, &mut self.context)
                    .expect("failed to define function");

                self.module.clear_context(&mut self.context);
            }
        }
    }
}

pub fn codegen(_typechecker_result: &TypecheckerResult, definitions: &[Definition]) -> Vec<u8> {
    let mut codegen = Codegen::new("main");

    for definition in definitions {
        codegen.visit_definition(definition);
    }

    codegen.finish()
}
