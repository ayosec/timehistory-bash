//! timehistory bash builtin

use bash_builtins::{builtin_metadata, Args, Builtin, BuiltinOptions, Result as BuiltinResult};
use std::io::{self, BufWriter, Write};

builtin_metadata!(
    name = "timehistory",
    try_create = TimeHistory::new,
    short_doc = "timehistory [-f fmt | -v | -j] [<n> | +<n>] | -R | -C | [-L limit] [-F fmt]",
    long_doc = "
        Displays information about the resources used by programs executed in
        the running shell.

        Options:
          -f FMT\tUse FMT as the format string for every history entry,
                \tinstead of the default value.
          -v\tUse the verbose format, similar to GNU time.
          -j\tPrint information as JSON format.
          -R\tRemove all entries in the history.
          -C\tShow the current configuration.
          -F\tChange the default format string.
          -L\tChange the history limit.

        Use '-f help' to get information about the formatting syntax.

        If <n> is given, it displays all information for a specific history
        entry. The number for every entry is printed with the %n specifier in
        the format string. If the number is prefixed with a plus symbol (+<n>)
        it is the offset from the end of the list ('+1' is the last entry).
    ",
);

mod format;
mod history;
mod ipc;
mod jsonext;
mod procs;

#[cfg(test)]
mod tests;

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

    #[opt = 'v']
    VerboseFormat,

    #[opt = 'j']
    Json,

    #[opt = 'R']
    Reset,

    #[opt = 'C']
    PrintConfig,

    #[opt = 'F']
    SetDefaultFormat(String),

    #[opt = 'L']
    SetLimit(usize),
}

enum Output {
    Format(String),
    Verbose,
    Json,
}

enum Action {
    List,
    Reset,
    ShowItem(usize),
}

impl TimeHistory {
    fn new() -> Result<TimeHistory, Box<dyn std::error::Error>> {
        if ipc::global_shared_buffer(Duration::from_millis(100)).is_none() {
            return Err("shared buffer unavailable".into());
        }

        let fn_replacements = procs::replace_functions()?;

        unsafe {
            history::OWNER_PID = libc::getpid();
        }

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

        let mut history = match crate::ipc::events::collect_events() {
            Some(history) => history,
            None => return Err(bash_builtins::Error::ExitCode(1)),
        };

        // Extract options from command-line.

        let mut exit_after_options = false;
        let mut output_format = None;
        let mut action = Action::List;

        macro_rules! set_format {
            ($($t:tt)+) => {{
                if output_format.is_some() {
                    bash_builtins::log::show_usage();
                    return Err(bash_builtins::Error::Usage);
                }

                output_format = Some(Output::$($t)+);
            }}
        }

        for opt in args.options() {
            match opt? {
                Opt::Format("help") => {
                    output.write_all(format::HELP)?;
                    exit_after_options = true;
                }

                Opt::Format(fmt) => set_format!(Format(fmt.to_owned())),

                Opt::VerboseFormat => set_format!(Verbose),

                Opt::Json => set_format!(Json),

                Opt::Reset => action = Action::Reset,

                Opt::PrintConfig => {
                    writeln!(
                        &mut output,
                        "-L {} -F {}",
                        history.size(),
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
                    history.set_size(l as usize);
                    exit_after_options = true;
                }
            }
        }

        if exit_after_options {
            args.finished()?;
            return Ok(());
        }

        // Check if the `<n>` argument is present, but only to replace the
        // `List` action.
        if matches!(action, Action::List) {
            if let Some(arg) = args.string_arguments().next() {
                let arg = arg?;
                let number = match arg.parse()? {
                    n if n > 0 && arg.starts_with('+') => history.offset_number(n),
                    n => n,
                };
                action = Action::ShowItem(number);
            }
        }

        args.finished()?;

        let format = match &output_format {
            None => Some(self.default_format.as_ref()),
            Some(Output::Format(f)) => Some(f.as_ref()),
            Some(Output::Verbose) => Some(include_str!("verbose.fmt")),
            Some(Output::Json) => None,
        };

        match (action, format) {
            (Action::List, None) => {
                let mut first = true;
                output.write_all(b"[\n")?;

                for entry in history.entries.iter().rev() {
                    if !std::mem::replace(&mut first, false) {
                        output.write_all(b",\n")?;
                    }

                    serde_json::to_writer(&mut output, entry)?;
                }

                output.write_all(b"\n]\n")?;
            }

            (Action::List, Some(fmt)) => {
                for entry in history.entries.iter().rev() {
                    format::render(entry, fmt, &mut output)?;
                    output.write_all(b"\n")?;
                }
            }

            (Action::Reset, _) => {
                history.entries.clear();
            }

            (Action::ShowItem(number), output_format) => {
                if let Some(entry) = history.entries.iter().find(|e| e.number == number) {
                    match output_format {
                        None => serde_json::to_writer(&mut output, entry)?,
                        Some(fmt) => format::render(entry, fmt, &mut output)?,
                    }

                    output.write_all(b"\n")?;
                }
            }
        }

        Ok(())
    }
}
