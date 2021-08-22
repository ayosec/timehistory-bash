//! timehistory bash builtin

use bash_builtins::{builtin_metadata, Args, Builtin, BuiltinOptions};
use bash_builtins::{Error::Usage, Result as BuiltinResult};
use std::io::{self, BufWriter, Write};

builtin_metadata!(
    name = "timehistory",
    try_create = TimeHistory::new,
    short_doc = "timehistory [-f FMT | -v | -j] [<n> | +<n>] | -s SET | -R",
    long_doc = "
        Displays information about the resources used by programs executed in
        the running shell.

        Options:
          -f FMT\tUse FMT as the format string for every history entry,
                \tinstead of the default value.
          -v\tUse the verbose format, similar to GNU time.
          -j\tPrint information as JSON format.
          -s SET\tChange the value of a setting. See below.
          -R\tRemove all entries in the history.

        If <n> is given, it displays information for a specific history entry.
        The number for every entry is printed with the %n specifier in the
        format string. If the number is prefixed with a plus symbol (+<n>) it
        is the offset from the end of the list ('+1' is the last entry).

        Format:
          Use '-f help' to get information about the formatting syntax.

        Settings:
          The following settings are available:

            format\tDefault format string.
            header\tShow a header with the labels of every resource.
            limit\tHistory limit.

          To change a setting, use '-s name=value', where 'name' is any of the
          previous values. Use one '-s' for every setting to change.

          To see the current values use '-c show'.
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

const DEFAULT_FORMAT: &str = "%n\\t%(time:%X)\\t%P\\t%e\\t%C";

struct TimeHistory {
    /// Default format to print history entries.
    default_format: String,

    /// Show header with field labels.
    show_header: bool,
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

    #[opt = 's']
    Setting(&'a str),

    #[cfg(feature = "option-for-panics")]
    #[opt = 'P']
    Panic,
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

        procs::replace_functions()?;

        unsafe {
            history::OWNER_PID = libc::getpid();
        }

        Ok(TimeHistory {
            default_format: DEFAULT_FORMAT.into(),
            show_header: false,
        })
    }
}

impl Builtin for TimeHistory {
    fn call(&mut self, args: &mut Args) -> BuiltinResult<()> {
        let stdout_handle = io::stdout();
        let mut output = BufWriter::new(stdout_handle.lock());

        let mut history = match crate::ipc::events::collect_events(true) {
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
                    return Err(Usage);
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

                Opt::Setting("show") => {
                    self.print_config(&mut output, &history)?;
                    exit_after_options = true;
                }

                Opt::Setting(setting) => {
                    let mut parts = setting.splitn(2, '=');
                    match (parts.next(), parts.next()) {
                        (Some("limit"), Some(value)) => {
                            history.set_size(value.parse()?);
                        }

                        (Some("header"), Some(value)) => {
                            self.show_header = value.parse()?;
                        }

                        (Some("format"), Some(value)) => {
                            self.default_format = if value.is_empty() {
                                DEFAULT_FORMAT.into()
                            } else {
                                value.to_owned()
                            };
                        }

                        (Some(name), _) => {
                            bash_builtins::error!("{}: invalid setting", name);
                            return Err(Usage);
                        }

                        _ => {
                            bash_builtins::error!("{}: missing value", setting);
                            return Err(Usage);
                        }
                    }

                    exit_after_options = true;
                }

                #[cfg(feature = "option-for-panics")]
                Opt::Panic => panic!("-P"),
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
            Some(Output::Verbose) => Some(include_str!("format/verbose.fmt")),
            Some(Output::Json) => None,
        };

        if self.show_header {
            if let Some(fmt) = &format {
                format::labels(fmt, &mut output)?;
                output.write_all(b"\n")?;
            } else {
                bash_builtins::warning!("header not available in JSON output.");
            }
        }

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

impl TimeHistory {
    fn print_config(&self, mut output: impl Write, history: &history::History) -> io::Result<()> {
        writeln!(
            &mut output,
            "format={}\n\
             header={}\n\
             limit={}",
            self.default_format,
            self.show_header,
            history.size(),
        )?;

        Ok(())
    }
}
