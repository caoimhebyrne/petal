use cranelift::{frontend, module::Module, object};
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
            self.visit_statement(statement);
        }

        self.builder.seal_block(entry_block);
        self.builder.finalize(self.module.target_config());
    }
}
