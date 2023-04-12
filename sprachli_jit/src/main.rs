use dynasmrt::{dynasm, AssemblyOffset, DynasmApi, DynasmLabelApi, ExecutableBuffer, Executor};

use std::io::Write;
use std::{io, mem, slice};

fn main() {
    let mut hello = DynReplacement::new("Hello World!".to_owned());
    assert!(!hello.call());
    hello.compile();
    assert!(hello.call());
}

type GreeterFn = extern "win64" fn() -> bool;

/// An Uncompiled instance contains the necessary info for calling the yet-to-compile code and
/// for compiling it
struct Uncompiled<T> {
    /// assembler for the trampoline which jumps to either a stub or the
    /// JIT-compiled implementation
    ops: dynasmrt::x64::Assembler,
    /// executor for the trampoline, so that it can be called
    executor: Executor,
    /// offset for the trampoline, so that it can be called
    offset: AssemblyOffset,
    /// data preserved for later access by the compiler
    data: T,
}

/// A Compiled instance contains the buffers for trampoline and compiled code, and a function
/// pointer to the trampoline through which the compiled code can be called
struct Compiled {
    /// the trampoline machine code
    _trampoline: ExecutableBuffer,
    /// the actual implementation machine code
    _impl: ExecutableBuffer,
    /// function pointer to the trampoline
    func: GreeterFn,
}

impl<T> Uncompiled<T> {
    fn new(data: T, stub: GreeterFn) -> Self {
        let mut ops = dynasmrt::x64::Assembler::new().unwrap();

        let offset = ops.offset();
        dynasm!(ops
            ; .arch x64
            ; mov rax, QWORD stub as _
            ; jmp rax
        );
        ops.commit().expect("commit");

        let executor = ops.reader();

        Self {
            ops,
            executor,
            offset,
            data,
        }
    }

    pub fn call(&self) -> bool {
        let buf = self.executor.lock();
        let ptr = buf.ptr(self.offset);
        // SAFETY: the pointer is indeed to a function and not preserved longer than the executor is locked
        let func: GreeterFn = unsafe { mem::transmute(ptr) };

        func()
    }

    pub fn compile<F>(mut self, assemble: F) -> Compiled
    where
        F: FnOnce(&mut dynasmrt::x64::Assembler, T) -> AssemblyOffset,
    {
        let (r#impl, func) = {
            let mut ops = dynasmrt::x64::Assembler::new().unwrap();

            let offset = assemble(&mut ops, self.data);

            ops.commit().expect("commit");
            let buf = ops.finalize().expect("finalize");

            let ptr = buf.ptr(offset);
            // SAFETY: the pointer is indeed to a function and not preserved longer than the buffer is kept around
            (buf, unsafe { mem::transmute::<_, GreeterFn>(ptr) })
        };

        let (trampoline, func) = {
            // ops.finalize() requires that no executor is still around
            drop(self.executor);

            self.ops
                .alter(|m| {
                    m.check_exact(self.offset)
                        .expect("offset must be the same as before");
                    dynasm!(m
                        ; .arch x64
                        ; mov rax, QWORD func as _
                        ; jmp rax
                    );
                })
                .expect("alter");

            self.ops.commit().expect("commit");
            let buf = self.ops.finalize().expect("finalize");

            let ptr = buf.ptr(self.offset);
            // SAFETY: the pointer is indeed to a function and not preserved longer than the buffer is kept around
            (buf, unsafe { mem::transmute::<_, GreeterFn>(ptr) })
        };

        Compiled::new(trampoline, r#impl, func)
    }
}

impl Compiled {
    fn new(trampoline: ExecutableBuffer, r#impl: ExecutableBuffer, func: GreeterFn) -> Self {
        Self {
            _trampoline: trampoline,
            _impl: r#impl,
            func,
        }
    }

    pub fn call(&self) -> bool {
        (self.func)()
    }
}

enum DynReplacement {
    Uncompiled(Uncompiled<String>),
    Compiling,
    Compiled(Compiled),
}

impl DynReplacement {
    fn new(string: String) -> Self {
        extern "win64" fn stub() -> bool {
            false
        }

        Self::Uncompiled(Uncompiled::new(string, stub))
    }

    pub fn call(&self) -> bool {
        use DynReplacement::*;

        match self {
            Uncompiled(uncompiled) => uncompiled.call(),
            Compiled(compiled) => compiled.call(),
            Compiling => panic!("call() while Compiling"),
        }
    }

    pub fn compile(&mut self) {
        use DynReplacement::*;

        let uncompiled = match mem::replace(self, Compiling) {
            Uncompiled(uncompiled) => uncompiled,
            old => {
                *self = old;
                panic!("compile");
            }
        };

        let compiled = uncompiled.compile(|ops, string| {
            dynasm!(ops
                ; .arch x64
                ; ->msg:
                ; .bytes string.as_bytes()
            );

            let offset = ops.offset();
            dynasm!(ops
                ; .arch x64
                ; lea rcx, [->msg]
                ; xor edx, edx
                ; mov dl, BYTE string.len() as _
                ; mov rax, QWORD print as _
                // no idea what the 0x28 here means, but it's somehow related to the win64 calling convention
                ; sub rsp, BYTE 0x28
                ; call rax
                ; add rsp, BYTE 0x28
                ; ret
            );

            offset
        });

        *self = Compiled(compiled);
    }
}

// a function with well-defined calling convention that can be called from generated machine code
pub extern "win64" fn print(buffer: *const u8, length: u64) -> bool {
    let mut result = true;

    result = result
        && io::stdout()
            .write_all(unsafe { slice::from_raw_parts(buffer, length as usize) })
            .is_ok();
    result = result && io::stdout().flush().is_ok();

    result
}
