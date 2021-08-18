//! timehistory bash builtin

use bash_builtins::{builtin_metadata, Args, Builtin, BuiltinOptions, Result as BuiltinResult};
use std::io::{self, BufWriter, Write};

builtin_metadata!(name = "timehistory", try_create = TimeHistory::new,);

mod format;
mod history;
mod ipc;
mod procs;

use std::time::Duration;

const DEFAULT_FORMAT: &str = "%n\t%P\t%e\t%C";

struct TimeHistory {
    /// Default format to print history entries.
    default_format: String,

    /// Replacements for libc functions.
    ///
    /// Stored to invoke the destructors when the builtin is removed.
    #[allow(dead_code)]
    fn_replacements: procs::Replacements,
}

#[derive(BuiltinOptions)]
enum Opt<'a> {
    #[opt = 'f']
    Format(&'a str),

    #[opt = 'F']
    SetDefaultFormat(String),
}

/// Action to execute after parsing options.
#[derive(PartialEq)]
enum Action {
    List,
    Exit,
}

impl TimeHistory {
    fn new() -> Result<TimeHistory, Box<dyn std::error::Error>> {
        if ipc::global_shared_buffer(Duration::from_millis(100)).is_none() {
            return Err("shared buffer unavailable".into());
        }

        let fn_replacements = procs::replace_functions()?;
        Ok(TimeHistory {
            default_format: DEFAULT_FORMAT.into(),
            fn_replacements,
        })
    }
}

impl Builtin for TimeHistory {
    fn call(&mut self, args: &mut Args) -> BuiltinResult<()> {
        let stdout_handle = io::stdout();
        let mut output = BufWriter::new(stdout_handle.lock());

        // Extract options from command-line.

        let mut action = Action::List;
        let mut format = None;

        for opt in args.options() {
            match opt? {
                Opt::Format("help") => {
                    output.write_all(format::HELP)?;
                    action = Action::Exit;
                }

                Opt::Format(fmt) => format = Some(fmt.to_owned()),

                Opt::SetDefaultFormat(fmt) => {
                    self.default_format = if fmt.is_empty() {
                        DEFAULT_FORMAT.into()
                    } else {
                        fmt
                    };

                    action = Action::Exit;
                }
            }
        }

        if action == Action::Exit {
            args.finished()?;
            return Ok(());
        }

        // Show history entries.

        let history = match history::HISTORY.try_lock() {
            Ok(l) => l,

            Err(e) => {
                bash_builtins::error!("history unavailable: {}", e);
                return Err(bash_builtins::Error::ExitCode(1));
            }
        };

        let format = format.as_ref().unwrap_or(&self.default_format);

        for entry in history.entries.iter().rev() {
            format::render(entry, format, &mut output)?;
            output.write_all(b"\n")?;
        }

        Ok(())
    }
}
