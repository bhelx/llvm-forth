use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::OptimizationLevel;
use inkwell::basic_block::BasicBlock;
use inkwell::values::FunctionValue;
use inkwell::IntPredicate;

use std::fs;
use std::collections::HashMap;

use crate::ast::Ast;

lalrpop_mod!(pub parser);

type MainFunc = unsafe extern "C" fn() -> i64;

struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    main_block: Option<BasicBlock<'ctx>>,
    function: Option<FunctionValue<'ctx>>,
    variables: HashMap<String, i64>,
    variable_addr: i64,
}

impl<'ctx> CodeGen<'ctx> {
    fn parse(&self, input: &str) -> Ast {
        parser::PhraseParser::new().parse(input).unwrap()
    }

    fn jit_compile_main(&self) -> Option<JitFunction<MainFunc>> {
        unsafe { self.execution_engine.get_function("main").ok() }
    }

    fn build_push(&mut self, stack: &str, value: i64) {
        let i64_type = self.context.i64_type();
        let push_fn = self.module.get_function("push").unwrap();
        let data_stack = self.module.get_global(stack).unwrap().as_pointer_value().into();
        let push_val = i64_type.const_int(value as u64, false).into();
        let args = &[data_stack, push_val];
        self.builder.build_call(push_fn, args, "");
    }

    fn build_stack_fn(&mut self, name: &str) {
        let func = self.module.get_function(name).unwrap();
        self.builder.build_call(func, &[], "");
    }

    fn compile_expr(&mut self, expr: Ast) {
        match expr {
            Ast::Push(i) => {
                self.build_push("dataStack", i);
            },
            Ast::Conditional{consequent: c, alternative: a} => {
                if let Some(function) = self.function {
                    let then_block = self.context.append_basic_block(function, "then");
                    let else_block = self.context.append_basic_block(function, "else");
                    let merge_block = self.context.append_basic_block(function, "merge");

                    let push_fn = self.module.get_function("pop").unwrap();
                    let data_stack = self.module.get_global("dataStack").unwrap().as_pointer_value().into();
                    let args = &[data_stack];
                    let popped = self.builder.build_call(push_fn, args, "");
                    let popped_val = popped.try_as_basic_value().left().unwrap().into_int_value();

                    let i64_type = self.context.i64_type();
                    let neg_one = -1;
                    let tru = i64_type.const_int(neg_one as u64, false);

                    let cond_cmp = self.builder.build_int_compare(IntPredicate::EQ, popped_val, tru, "cond_cmp");

                    self.builder.build_conditional_branch(cond_cmp, then_block, else_block);

                    self.builder.position_at_end(then_block);
                    self.compile_expr(*c);
                    self.builder.build_unconditional_branch(merge_block);

                    self.builder.position_at_end(else_block);
                    if let Some(alt) = a {
                        self.compile_expr(*alt);
                    }
                    self.builder.build_unconditional_branch(merge_block);

                    self.builder.position_at_end(merge_block);
                    self.builder.build_phi(i64_type, "iftmp");
                }
            },
            Ast::Word(word) => {
                match &word[..] {
                    "+" => self.build_stack_fn("plus"),
                    "nand" => self.build_stack_fn("nand"),
                    "@" => self.build_stack_fn("fetch"),
                    "!" => self.build_stack_fn("store"),
                    ">r" => self.build_stack_fn("rPush"),
                    "r>" => self.build_stack_fn("rPop"),
                    w => {
                        match self.module.get_function(w) {
                            Some(fun) => {
                                self.builder.build_call(fun, &[], "");
                            },
                            None => {
                                match self.variables.get(w) {
                                    Some(addr) => {
                                        self.build_push("dataStack", *addr - 1000);
                                    },
                                    None => {}
                                }
                            }
                        }
                    }
                }
            },
            Ast::Phrase(items) => {
                for item in items {
                    self.compile_expr(item)
                }
            },
            Ast::Definition(phrase) => {
                if let Ast::Phrase(items) = *phrase {
                    if let Ast::Word(name) = items.get(0).unwrap() {
                        let mut items = items.clone();
                        items.remove(0);
                        let void_type = self.context.void_type();
                        let fn_type = void_type.fn_type(&[], false);
                        let function = self.module.add_function(name, fn_type, None);
                        self.function = Some(function);
                        let basic_block = self.context.append_basic_block(function, "entry");
                        self.builder.position_at_end(basic_block);
                        for item in items {
                            self.compile_expr(item);
                        }
                        self.builder.build_return(None);
                        self.builder.position_at_end(self.main_block.unwrap());
                        self.function = None;
                    }
                }
            },
            Ast::Variable(name) => {
                self.variables.insert(name, self.variable_addr);
                self.variable_addr = self.variable_addr + 1;
            },
            _ => {
            }
        }
    }

    fn compile_module(&mut self, code: &str) -> Option<String> {
        let i64_type = self.context.i64_type();
        let init_globals = self.module.get_function("initGlobals").unwrap();
        let main_fn_type = i64_type.fn_type(&[], false);
        let function = self.module.add_function("main", main_fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        // set pointer to this block so we can get back to it
        self.main_block = Some(basic_block);

        self.builder.position_at_end(basic_block);

        self.builder.build_call(init_globals, &[], "");

        let expr = self.parse("
: true -1 ;
: false 0 ;

variable  temp
: swap   >r temp ! r> temp @ ;
: over   >r temp ! temp @ r> temp @ ;
: rot    >r swap r> swap ;

: dup    temp ! temp @ temp @ ;
: 2dup   over over ;
: ?dup   temp ! temp @ if temp @ temp @ then ;

: nip    >r temp ! r> ;

: invert   -1 nand ;
: negate   invert 1 + ;
: -        negate + ;

: 1+   1 + ;
: 1-   -1 + ;
: +!   dup >r @ + r> ! ;
: 0=   if 0 else -1 then ;
: =    - 0= ;
: <>   = 0= ;

: or   invert swap invert nand ;
: xor   2dup nand 1+ dup + + + ;
: and   nand invert ;
: 2*    dup + ;

: <   2dup xor 0< if drop 0< else - 0< then ;
: u<   2dup xor 0< if nip 0< else - 0< then ;
: >   swap < ;
: u>   swap u> ;

: c@   @ 255 and ;
: c!   dup >r @ 65280 and + r> ! ;
        ");
        self.compile_expr(expr);

        let expr = self.parse(code); 
        self.compile_expr(expr);

        let data_stack = self.module.get_global("dataStack").unwrap().as_pointer_value().into();
        let peek_stack = self.module.get_function("peek").unwrap();
        let peeked = self.builder.build_call(peek_stack, &[data_stack], "peeked");
        let peeked_val = peeked.try_as_basic_value().left().unwrap();

        self.builder.build_return(Some(&peeked_val));

        unsafe { Some(self.module.print_to_string().to_string()) }
    }

}

pub struct Compiler<'ctx> {
    codegen: CodeGen<'ctx>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Compiler<'ctx> {
        // load runtime
        let machine_ir = fs::read("./machine.bc").expect("Something went wrong reading the file");
        let buffer = MemoryBuffer::create_from_memory_range(&machine_ir, "kernel");
        let machine = Module::parse_bitcode_from_buffer(&buffer, &context).unwrap();
        let execution_engine = machine.create_jit_execution_engine(OptimizationLevel::None).unwrap();
        let mut codegen = CodeGen {
            context: &context,
            module: machine,
            builder: context.create_builder(),
            execution_engine,
            main_block: None,
            function: None,
            variables: HashMap::new(),
            variable_addr: 1000,
        };

        Compiler {
            codegen
        }
    }

    pub fn compile_and_run(&mut self, code: &str) -> String {
        println!("{}", self.codegen.compile_module(code).unwrap());
        let main = self.codegen.jit_compile_main().ok_or("Unable to JIT compile `main`").unwrap();
        unsafe {
            main.call().to_string()
        }

    }
}