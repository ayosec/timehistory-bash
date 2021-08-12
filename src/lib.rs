//! timehistory bash builtin

use bash_builtins::{builtin_metadata, Builtin, Result, Args};

builtin_metadata!(
    name =  "timehistory",
    create =  TimeHistory::default,
);

#[derive(Default)]
struct TimeHistory;

impl Builtin for TimeHistory {
    fn call(&mut self, args: &mut Args) -> Result<()> {
        args.no_options()?;
        args.finished()?;
        Ok(())
    }
}
