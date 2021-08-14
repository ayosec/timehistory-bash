//! timehistory bash builtin

use bash_builtins::{builtin_metadata, Args, Builtin, Result as BuiltinResult};

builtin_metadata!(name = "timehistory", try_create = TimeHistory::new,);

mod funcwrappers;
mod ipc;

use std::time::Duration;

#[allow(dead_code)]
struct TimeHistory {
    fn_replacements: funcwrappers::Replacements,
}

impl TimeHistory {
    fn new() -> Result<TimeHistory, Box<dyn std::error::Error>> {
        if ipc::global_shared_buffer(Duration::from_millis(100)).is_none() {
            return Err("shared buffer unavailable".into());
        }

        let fn_replacements = funcwrappers::replace_functions()?;
        Ok(TimeHistory { fn_replacements })
    }
}

impl Builtin for TimeHistory {
    fn call(&mut self, args: &mut Args) -> BuiltinResult<()> {
        args.no_options()?;
        args.finished()?;
        Ok(())
    }
}
