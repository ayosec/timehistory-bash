//! timehistory bash builtin

use bash_builtins::{builtin_metadata, Args, Builtin, Result as BuiltinResult};

builtin_metadata!(name = "timehistory", try_create = TimeHistory::new,);

mod format;
mod history;
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

        let history = match history::HISTORY.try_lock() {
            Ok(l) => l,

            Err(e) => {
                bash_builtins::error!("history unavailable: {}", e);
                return Err(bash_builtins::Error::ExitCode(1));
            }
        };

        for entry in history.entries.iter().rev() {
            println!("{} {:?} {:?}", entry.unique_id, entry.pid, entry.args);
            match &entry.state {
                history::State::Running { .. } => println!("running"),
                history::State::Finished {
                    running_time,
                    rusage,
                    status,
                } => {
                    println!(
                        "\t{:?}\n\t{}\n\tstatus={} maxrss={}",
                        running_time, entry.start_time, status, rusage.ru_maxrss
                    )
                }
            }
        }

        Ok(())
    }
}
