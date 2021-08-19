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
static mut REPLACEMENTS: Option<[Replacement; 2]> = None;

/// Replace `waitpid` and `execve` functions.
pub(crate) fn replace_functions() -> Result<(), Box<dyn std::error::Error>> {
    let main_program = ObjectFile::open_main_program()?;

    unsafe {
        // Register a function to restore the original addresses when
        // this shared object is unloaded by dlclose().
        if libc::atexit(restore_functions) != 0 {
            return Err(std::io::Error::last_os_error().into());
        }

        // Replace waitpid and execve PLT entries.
        let waitpid_fn = main_program.replace("waitpid", waitpid::waitpid_wrapper as *const _)?;
        let execve_fn = main_program.replace("execve", execve::execve_wrapper as *const _)?;

        EXECVE_FN = Some(mem::transmute(execve_fn.original_address()));

        REPLACEMENTS = Some([waitpid_fn, execve_fn]);
    }

    Ok(())
}

/// Restore original libc functions.
extern "C" fn restore_functions() {
    unsafe {
        drop(REPLACEMENTS.take());
    }
}
