//! timehistory bash builtin

use bash_builtins::{builtin_metadata, Args, Builtin, BuiltinOptions, Result as BuiltinResult};
use std::io::{self, BufWriter, Write};

builtin_metadata!(name = "timehistory", try_create = TimeHistory::new,);

mod format;
mod history;
mod ipc;
mod procs;

use std::borrow::Cow;
use std::time::Duration;

const DEFAULT_FORMAT: &str = "%n\t%P\t%(elapsed)\t%C";

#[allow(dead_code)]
struct TimeHistory {
    fn_replacements: procs::Replacements,
}

#[derive(BuiltinOptions)]
enum Opt {
    #[opt = 'f']
    Format(String),
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
        // Extract options from command-line.

        let mut format = Cow::from(DEFAULT_FORMAT);

        for opt in args.options() {
            match opt? {
                Opt::Format(f) => format = f.into(),
            }
        }

        args.finished()?;

        // Show history entries.

        let history = match history::HISTORY.try_lock() {
            Ok(l) => l,

            Err(e) => {
                bash_builtins::error!("history unavailable: {}", e);
                return Err(bash_builtins::Error::ExitCode(1));
            }
        };

        let stdout_handle = io::stdout();
        let mut output = BufWriter::new(stdout_handle.lock());

        for entry in history.entries.iter().rev() {
            format::render(entry, &format, &mut output)?;
            output.write_all(b"\n")?;
        }

        Ok(())
    }
}
