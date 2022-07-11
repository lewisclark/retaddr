#![feature(naked_functions, asm_sym)]

mod winmodule;

use std::arch::asm;

// Protects a given function by creating a naked asm wrapper function
// which in turn will call the given function argument.
#[macro_export]
macro_rules! wrap {
    ( $wrapper:ident, $ret:ty, $callee:ident ) => {
        #[naked]
        unsafe extern "cdecl" fn $wrapper() -> $ret {
            asm!(
                "call {c}",
                "pushad",
                "call {f}",
                "popad",
                "ret",
                c = sym check_retaddr,
                f = sym $callee,
                options(noreturn),
            )
        }
    };
}

#[allow(dead_code)]
#[naked]
unsafe extern "cdecl" fn check_retaddr() {
    asm!(
        "push [esp]",
        "call {f}",
        "ret 4",
        f = sym check_retaddr_internal,
        options(noreturn)
    );
}

unsafe extern "cdecl" fn check_retaddr_internal(retaddr: usize) {
    let exe_module = winmodule::WinModules::new().unwrap().next().unwrap();
    let valid_min = exe_module.modBaseAddr as usize;
    let valid_max = valid_min + exe_module.modBaseSize as usize;

    if retaddr < valid_min || retaddr > valid_max {
        panic!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn protectee() {}

    wrap!(protector, (), protectee);

    #[test]
    fn works() {
        unsafe { protector() };
    }
}
