use std::{mem::transmute, process::exit};

use crate::types;
use cranelift::prelude::{AbiParam, FunctionBuilder, InstBuilder, Signature, Value, types::I64};
use cranelift_jit::JITBuilder;
use cranelift_module::{Linkage, Module, ModuleError};
use thiserror::Error;

type JITFunction = extern "C" fn(*mut types::ValueStack) -> ();

#[derive(Debug, Error)]
pub(crate) enum CompilerError {
    #[error("Module (cranelift) Error: {0}")]
    ModuleError(#[from] ModuleError),
}

impl types::ClacState {
    pub(crate) fn compile_function(
        &mut self,
        name: &str,
        line: &[types::Instr],
    ) -> Result<JITFunction, CompilerError> {
        let types::JITState {
            ctx,
            fbctx,
            module,
            imports: types::Imports { pushfunc, popfunc },
        } = &mut self.jit;

        module.clear_context(ctx);

        ctx.func.signature = Signature {
            params: vec![AbiParam::new(module.isa().pointer_type())], // *mut ValueStack
            returns: vec![],
            call_conv: module.isa().default_call_conv(),
        };

        let popper = module.declare_func_in_func(*popfunc, &mut ctx.func);
        let pusher = module.declare_func_in_func(*pushfunc, &mut ctx.func);

        let mut bu = FunctionBuilder::new(&mut ctx.func, fbctx);

        let entry = bu.create_block();
        bu.append_block_params_for_function_params(entry);
        bu.switch_to_block(entry);
        bu.seal_block(entry);

        // Idea:
        //
        // 2 levels of stack
        // there is the REAL stack (passed in pointer)
        // and also a build/function stack (*mut ClacStack)
        //
        // Before if statements/control flow, we commit/flush the build function stack, which means pushing everything onto the build function stack onto the real stack.
        // if we get to the final block, then we geneate instructions to push all of the build stack onto the REAL stack.
        // must also flush before Pick
        //
        // then every function is fn(*mut ClacStack) -> ()
        //
        let stack = bu.block_params(entry)[0];

        let mut tmp: Vec<Value> = Vec::new();

        let flush = |tmp: &mut Vec<Value>, bu: &mut FunctionBuilder| {
            for val in &*tmp {
                bu.ins().call(pusher, &[stack, *val]);
            }

            tmp.clear();
        };

        // let mut xpush = |tmp: &mut Vec<Value>, bu: &mut FunctionBuilder| {
        //     tmp.pop().unwrap_or_else(|| {
        //         let call_instr = bu.ins().call(popper, &[stack]);
        //     })
        // };

        let mut xpop = |tmp: &mut Vec<Value>, bu: &mut FunctionBuilder| {
            tmp.pop().unwrap_or_else(|| {
                let call_instr = bu.ins().call(popper, &[stack]);
                let results = bu.inst_results(call_instr);
                results[0]
            })
        };

        for inst in line {
            use types::Instr;
            match inst {
                Instr::Literal(n) => {
                    let out = bu.ins().iconst(I64, *n);
                    tmp.push(out);
                }
                it @ (Instr::Add | Instr::Sub | Instr::Mul | Instr::Div | Instr::Rem) => {
                    let b = xpop(&mut tmp, &mut bu);
                    let a = xpop(&mut tmp, &mut bu);

                    tmp.push(match it {
                        Instr::Add => bu.ins().iadd(a, b),
                        Instr::Sub => bu.ins().isub(a, b),
                        Instr::Mul => bu.ins().imul(a, b),
                        Instr::Div => bu.ins().sdiv(a, b),
                        Instr::Rem => bu.ins().srem(a, b),
                        _ => unreachable!(),
                    });
                }
                Instr::Drop => {
                    xpop(&mut tmp, &mut bu);
                }
                _ => unimplemented!(),
            }
        }

        flush(&mut tmp, &mut bu);

        let _ret = bu.ins().return_(&[]);

        bu.seal_all_blocks(); // FIXME: investigate
        bu.finalize();

        println!("{}", ctx.func.display());

        let id = module.declare_function(name, Linkage::Local, &ctx.func.signature)?;
        module.define_function(id, ctx)?;

        module.finalize_definitions()?;

        let fun = module.get_finalized_function(id);
        println!("JIT compiled function = {:?}", fun);

        Ok(unsafe { transmute(fun) })
    }
}
