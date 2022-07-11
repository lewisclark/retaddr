#![feature(naked_functions, asm_sym)]

mod winmodule;

/// Protects a given function by creating a wrapper around it.
///
/// * `wrapper` - the name of the new function that will wrap the given/protected function
/// * `ret` - the return type of the protected function
/// * `callee` - the protected function
/// * `check_fn` - the callback function that is called to validate the return address. Called with
/// a single argument, the return address as a usize. Must be `extern "cdecl"`.
#[macro_export]
macro_rules! wrap {
    ( $wrapper:ident, $ret:ty, $callee:ident, $check_fn:ident ) => {
        #[naked]
        unsafe extern "cdecl" fn $wrapper() -> $ret {
            std::arch::asm!(
                "push [esp]",
                "call {c}",
                "add esp, 4",
                "pushad",
                "call {f}",
                "popad",
                "ret",
                c = sym $check_fn,
                f = sym $callee,
                options(noreturn),
            )
        }
    };
}

/// A check function which can be used with the wrap! macro.
/// Determines if the return address is within the first module of the application and
/// panics if it isn't.
pub extern "cdecl" fn check_module_bounds_and_panic(retaddr: usize) {
    let exe_module = winmodule::WinModules::new().unwrap().next().unwrap();

    let min = exe_module.modBaseAddr as usize;
    let max = min + exe_module.modBaseSize as usize;

    if retaddr < min || retaddr > max {
        panic!("return address out of valid bounds");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn protectee1() {}
    fn check_fn1(_retaddr: usize) {}
    wrap!(protector1, (), protectee1, check_fn1);

    fn protectee2() {}
    fn check_fn2(_retaddr: usize) {}
    wrap!(protector2, (), protectee2, check_fn2);

    #[test]
    fn works() {
        unsafe { protector1() };
        unsafe { protector2() };
    }
}
