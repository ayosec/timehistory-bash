//! timehistory bash builtin

use bash_builtins::{builtin_metadata, Args, Builtin, Result as BuiltinResult};

builtin_metadata!(name = "timehistory", try_create = TimeHistory::new,);

mod ipc;
mod procs;

use std::time::Duration;

#[allow(dead_code)]
struct TimeHistory {
    fn_replacements: procs::Replacements,
}

impl TimeHistory {
    fn new() -> Result<TimeHistory, Box<dyn std::error::Error>> {
        if ipc::global_shared_buffer(Duration::from_millis(100)).is_none() {
            return Err("shared buffer unavailable".into());
        }

        let fn_replacements = procs::replace_functions()?;
        Ok(TimeHistory { fn_replacements })
    }
}

impl Builtin for TimeHistory {
    fn call(&mut self, args: &mut Args) -> BuiltinResult<()> {
        args.no_options()?;
        args.finished()?;

        let mut shared_buffer = match crate::ipc::global_shared_buffer(Duration::from_secs(1)) {
            Some(sb) => sb,
            None => {
                bash_builtins::error!("shared buffer unavailable");
                return Err(bash_builtins::Error::ExitCode(1));
            }
        };

        for event in ipc::events::ExecEvent::parse(shared_buffer.input()) {
            println!(
                "{} {} {} {:?}",
                event.pid, event.start_time.tv_sec, event.start_time.tv_nsec, event.args
            );
        }

        shared_buffer.clear();

        Ok(())
    }
}
