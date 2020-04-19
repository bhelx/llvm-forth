## LLVM-Forth

This is an attempt at writing an LLVM frontend in Rust. It uses [TheDan64/inkwell](https://github.com/TheDan64/inkwell) for LLVM API bindings and [lalrpop](https://github.com/lalrpop/lalrpop) for parsing code. I've chosen a small Forth to implement as the syntax and operational semantics of the language are so simple. It's not anything near a complete spec and all errors are ignored. I just wanted to walk through the whole process.

### Using

Make sure you have LLVM 9 and the latest Rust installed.

```bash
# Prints out the LLVM IR module and executes the code in src/main
cargo run
```

### Architecture

The system is composed of a few layers of abstraction.

#### machine.ll

First, all system functions are implemented in [machine.ll](machine.ll). This contains the machine's core datastructures and `7` Forth words implemented directly in LLVM IR:

* `drop`
* `@`
* `!`
* `r>`
* `>r`
* `nand`
* `+`

The compiler loads the assembled bitcode (`machine.bc`) into an LLVM module object then implements the rest of your program there. The machine can be recompiled using `llvm-as`:

```
llvm-as machine.ll
```

From these 7 operations, all other operations can be bootstrapped in Forth.

#### kernel.fth

Next, the "kernel" is written in Forth and is borrowed from `nybbleForth`. It implements the rest of the dictionary from the 7 words above:

```forth
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
```

> Copied from: https://github.com/larsbrinkhoff/nybbleForth/blob/master/forth/kernel.fth

The kernel is parsed and compiled into the module created by loading machine.ll.

#### User Code

Finally your code is parsed and compiled into the module which can be jit compiled and executed, or can spit out LLVM IR to be compiled by gcc.