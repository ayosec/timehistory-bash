//! Wrappers for libc functions.

use plthook::{ObjectFile, Replacement};
use std::mem;
use std::os::raw::{c_char, c_int};

mod execve;
mod waitpid;

/// Function signature for execve().
type ExecveFn = extern "C" fn(*const c_char, *const *const c_char, *const *const c_char) -> c_int;

/// Address of the original `execve`.
static mut EXECVE_FN: Option<ExecveFn> = None;

/// Replacements of the original libc functions.
#[allow(dead_code)]
pub(crate) struct Replacements {
    waitpid_fn: Replacement,
    execve_fn: Replacement,
}

/// Replace `waitpid` and `execve` functions.
pub(crate) fn replace_functions() -> Result<Replacements, Box<dyn std::error::Error>> {
    let main_program = ObjectFile::open_main_program()?;

    unsafe {
        let waitpid_fn = main_program.replace("waitpid", waitpid::waitpid_wrapper as *const _)?;
        let execve_fn = main_program.replace("execve", execve::execve_wrapper as *const _)?;

        EXECVE_FN = Some(mem::transmute(execve_fn.original_address()));

        Ok(Replacements {
            waitpid_fn,
            execve_fn,
        })
    }
}
