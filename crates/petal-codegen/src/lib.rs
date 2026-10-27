use cranelift::{codegen::isa::CallConv, prelude::*};
use cranelift_object::{ObjectBuilder, ObjectModule};
use petal_ast::Definition;
use petal_typecheck::TypecheckerResult;

struct Codegen {
    call_conv: CallConv,
    object_module: ObjectModule,
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

        let isa_builder = cranelift_native::builder().expect("host machine is not supported by cranelift");

        let isa = isa_builder
            .finish(settings::Flags::new(flags_builder))
            .expect("failed to build isa");

        let call_conv = isa.default_call_conv();

        let object_builder = ObjectBuilder::new(isa, object_name, cranelift_module::default_libcall_names())
            .expect("failed to create objectbuilder");

        let object_module = ObjectModule::new(object_builder);

        Self {
            call_conv,
            object_module,
        }
    }

    /// Finishes compilation by returning the bytes of the final object file.
    fn finish(self) -> Vec<u8> {
        let product = self.object_module.finish();
        product.emit().expect("failed to emit object")
    }
}

// https://github.com/bytecodealliance/cranelift-jit-demo/blob/main/src/jit.rs
pub fn codegen(_typechecker_result: &TypecheckerResult, _definitions: &[Definition]) -> Vec<u8> {
    let codegen = Codegen::new("main");
    codegen.finish()
}
