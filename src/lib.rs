//! timehistory bash builtin

use bash_builtins::{builtin_metadata, Args, Builtin, BuiltinOptions, Result as BuiltinResult};
use std::io::{self, BufWriter, Write};

builtin_metadata!(name = "timehistory", try_create = TimeHistory::new,);

mod format;
mod history;
mod ipc;
mod procs;

#[cfg(test)]
mod tests;

use std::sync::MutexGuard;
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

    #[opt = 'C']
    PrintConfig,

    #[opt = 'F']
    SetDefaultFormat(String),

    #[opt = 'L']
    SetLimit(usize),
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

        let mut exit_after_options = false;
        let mut format = None;

        for opt in args.options() {
            match opt? {
                Opt::Format("help") => {
                    output.write_all(format::HELP)?;
                    exit_after_options = true;
                }

                Opt::Format(fmt) => format = Some(fmt.to_owned()),

                Opt::PrintConfig => {
                    writeln!(
                        &mut output,
                        "-L {} -F {}",
                        history()?.size(),
                        format::EscapeArgument(self.default_format.as_bytes())
                    )?;

                    exit_after_options = true;
                }

                Opt::SetDefaultFormat(fmt) => {
                    self.default_format = if fmt.is_empty() {
                        DEFAULT_FORMAT.into()
                    } else {
                        fmt
                    };

                    exit_after_options = true;
                }

                Opt::SetLimit(l) => {
                    history()?.set_size(l as usize);
                    exit_after_options = true;
                }
            }
        }

        if exit_after_options {
            args.finished()?;
            return Ok(());
        }

        // Show history entries.
        let history = history()?;
        let format = format.as_ref().unwrap_or(&self.default_format);

        for entry in history.entries.iter().rev() {
            format::render(entry, format, &mut output)?;
            output.write_all(b"\n")?;
        }

        Ok(())
    }
}

fn history() -> Result<MutexGuard<'static, history::History>, bash_builtins::Error> {
    history::HISTORY.try_lock().map_err(|e| {
        bash_builtins::error!("history unavailable: {}", e);
        bash_builtins::Error::ExitCode(1)
    })
}
