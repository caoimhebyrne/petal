use cranelift::{codegen::ir::InstBuilder, frontend, module::Module, object};
use petal_ast::Block;

pub(crate) struct FunctionCodegen<'a> {
    pub(crate) builder: frontend::FunctionBuilder<'a>,
    pub(crate) module: &'a mut object::ObjectModule,
}

impl FunctionCodegen<'_> {
    pub(crate) fn build(mut self, block: &Block) {
        let entry_block = self.builder.create_block();
        self.builder.append_block_params_for_function_params(entry_block);
        self.builder.switch_to_block(entry_block);

        for statement in &block.statements {
            // Anything after a terminator (e.g. `return`) is unreachable, and cranelift does not allow instructions to
            // be added to a block that has already been terminated.
            if self.is_terminated() {
                break;
            }

            self.visit_statement(statement);
        }

        // Void functions are allowed to omit a trailing `return`. The typechecker ensures that non-void functions
        // always return a value, so this will never be reached for them.
        if !self.is_terminated() {
            self.builder.ins().return_(&[]);
        }

        self.builder.seal_block(entry_block);
        self.builder.finalize(self.module.target_config());
    }

    /// Return whether the current block ends in a terminator (e.g. `return`).
    fn is_terminated(&self) -> bool {
        let Some(block) = self.builder.current_block() else {
            return false;
        };

        self.builder
            .func
            .layout
            .last_inst(block)
            .is_some_and(|inst| self.builder.func.dfg.insts[inst].opcode().is_terminator())
    }
}
