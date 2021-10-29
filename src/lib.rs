//! timehistory bash builtin

use bash_builtins::{builtin_metadata, variables, warning, Args, Builtin, BuiltinOptions};
use bash_builtins::{Error::Usage, Result as BuiltinResult};

use std::borrow::Cow;
use std::io::{self, BufWriter, Write};

builtin_metadata!(
    name = "timehistory",
    try_create = TimeHistory::new,
    short_doc = "timehistory [-f FMT | -v | -j] [<n> | +<n>] | -s | -R",
    long_doc = "
        Displays information about the resources used by programs executed in
        the running shell.

        Options:
          -f FMT\tUse FMT as the format string for every history entry,
                \tinstead of the default value.
          -v\tUse the verbose format, similar to GNU time.
          -j\tPrint information as JSON format.
          -s\tPrint the current configuration settings.
          -R\tRemove all entries in the history.

        If <n> is given, it displays information for a specific history entry.
        The number for every entry is printed with the %n specifier in the
        format string. If the number is prefixed with a plus symbol (+<n>) it
        is the offset from the end of the list ('+1' is the last entry).

        Format:
          Use '-f help' to get information about the formatting syntax.

        Settings:
          The following shell variables can be used to change the configuration:

            TIMEHISTORY_FORMAT          Default format string.
            TIMEHISTORY_LIMIT           History limit.
            TIMEHISTORY_CMDLINE_LIMIT   Number of bytes to copy from the
                                        command line.
    ",
);

mod bytetables;
mod format;
mod history;
mod ipc;
mod jsonext;
mod procs;

#[cfg(test)]
mod tests;

use std::time::Duration;

const DEFAULT_FORMAT: &str = "[header,table]%n\\t%(time:%X)\\t%P\\t%e\\t%C";

/// Shell variable to set the format string.
const SHELL_VAR_FORMAT: &str = "TIMEHISTORY_FORMAT";

/// Shell variable to set the history limit.
const SHELL_VAR_LIMIT: &str = "TIMEHISTORY_LIMIT";

/// Shell variable to set the command line limit.
const SHELL_VAR_CMDLINE_LIMIT: &str = "TIMEHISTORY_CMDLINE_LIMIT";

struct TimeHistory;

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
    Setting(Option<&'a str>),

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

        variables::bind(SHELL_VAR_LIMIT, history::LimitVariable)?;
        variables::bind(SHELL_VAR_CMDLINE_LIMIT, ipc::CmdLineLimitVariable)?;

        procs::replace_functions()?;

        unsafe {
            history::OWNER_PID = libc::getpid();
        }

        Ok(TimeHistory)
    }
}

impl Builtin for TimeHistory {
    fn call(&mut self, args: &mut Args) -> BuiltinResult<()> {
        let mut table_writer;
        let stdout_handle = io::stdout();
        let mut output = &mut BufWriter::new(stdout_handle.lock()) as &mut dyn Write;

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

                Opt::Setting(None) => {
                    self.print_config(
                        &mut output,
                        &history,
                        ipc::global_shared_buffer(Duration::from_millis(100))
                            .map(|buf| buf.max_cmdline()),
                    )?;
                    exit_after_options = true;
                }

                Opt::Setting(Some(setting)) => {
                    warning!("-s is deprecated. Use the shell variables to change the settings");

                    let mut parts = setting.splitn(2, '=');
                    match (parts.next(), parts.next()) {
                        (Some("limit"), Some(value)) => {
                            history.set_size(value.parse()?);
                        }

                        (Some("format"), Some(value)) => {
                            variables::set(SHELL_VAR_FORMAT, value)?;
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
            None => Some(Self::default_format()),
            Some(Output::Format(f)) => Some(Cow::Borrowed(f.as_ref())),
            Some(Output::Verbose) => Some(include_str!("format/verbose.fmt").into()),
            Some(Output::Json) => None,
        };

        let format = format.as_deref().map(format::FormatOptions::parse);

        // Render output as a table.
        if let Some(options) = &format {
            if options.table {
                table_writer = format::TableWriter::new(output);
                output = &mut table_writer as &mut dyn Write;
            }

            if options.header {
                format::labels(options.format, &mut output)?;
                output.write_all(b"\n")?;
            }
        }

        match (action, format.map(|f| f.format)) {
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

        output.flush()?;

        Ok(())
    }
}

impl TimeHistory {
    fn print_config(
        &self,
        mut output: impl Write,
        history: &history::History,
        max_cmdline: Option<usize>,
    ) -> io::Result<()> {
        write!(
            &mut output,
            "\
             TIMEHISTORY_FORMAT        = {}\n\
             TIMEHISTORY_LIMIT         = {}\n\
            ",
            Self::default_format(),
            history.size(),
        )?;

        if let Some(max_cmdline) = max_cmdline {
            writeln!(&mut output, "TIMEHISTORY_CMDLINE_LIMIT = {}", max_cmdline)?;
        }

        Ok(())
    }

    fn default_format() -> Cow<'static, str> {
        variables::find_as_string(SHELL_VAR_FORMAT)
            .and_then(|s| s.into_string().ok().map(Cow::Owned))
            .unwrap_or_else(|| DEFAULT_FORMAT.into())
    }
}
