use super::EscapeArgument;
use crate::history::{Entry, State};
use chrono::{Local, TimeZone};
use std::ffi::OsString;
use std::time::Duration;

fn format_entry<F>(format: &str, mut f: F) -> (Entry, String)
where
    F: FnMut(&mut Entry),
{
    let mut entry = Entry {
        number: 1234,
        pid: 10000,
        start_time: Local.timestamp(1000000000, 9999),
        args: ["/bin/ls", "ls", "F"].iter().map(OsString::from).collect(),
        state: State::Running {
            start: libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
    };

    f(&mut entry);

    let mut output = Vec::new();
    super::render(&entry, format, &mut output).unwrap();
    (entry, String::from_utf8(output).unwrap())
}

#[test]
fn simple_specs() {
    assert_eq!(format_entry("%n pid=%(pid)", |_| ()).1, "1234 pid=10000");

    assert_eq!(format_entry("%n %C", |_| ()).1, "1234 ls F");

    assert_eq!(
        format_entry("%e %E %u", |entry| {
            entry.state = State::Finished {
                running_time: Some(Duration::from_millis(1801)),
                status: 0,
                rusage: unsafe { std::mem::zeroed() },
            }
        })
        .1,
        "1.801 0:01.801 1801000"
    );

    assert_eq!(
        format_entry("%e %E %u", |entry| {
            entry.state = State::Finished {
                running_time: Some(Duration::from_millis(7_500_301)),
                status: 0,
                rusage: unsafe { std::mem::zeroed() },
            }
        })
        .1,
        "7500.301 2:03:00 7500301000"
    );
}

#[test]
fn show_cpu_usage() {
    let items = [(15, "1.50"), (900, "90")];
    for (stime, pcent) in items {
        let rusage = unsafe {
            let mut r: libc::rusage = std::mem::zeroed();
            r.ru_stime.tv_sec = stime;
            r
        };

        assert_eq!(
            format_entry("%P", |entry| {
                entry.state = State::Finished {
                    running_time: Some(Duration::from_secs(1000)),
                    status: 0,
                    rusage,
                }
            })
            .1,
            format!("{}%", pcent)
        );
    }
}

#[test]
fn literal_chars() {
    assert_eq!(
        format_entry(r#"%% \n \e \t \\ \K \u{221e}"#, |_| ()).1,
        "% \n \x1b \t \\ \\K \u{221e}"
    );
}

#[test]
fn format_time() {
    let (entry, output) = format_entry("start at = %(time:%F %X)!", |_| ());

    let time = entry.start_time.format("%F %X");
    assert_eq!(output, format!("start at = {}!", time));
}

#[test]
fn keep_invalid_specs() {
    assert_eq!(
        format_entry("%(pid)%(piδ%(p%n%(pi", |_| ()).1,
        "10000%(piδ%(p1234%(pi"
    );
    assert_eq!(format_entry("%nn%(time:)%(time:", |_| ()).1, "1234n%(time:");
}

#[test]
fn escape_strings() {
    assert_eq!(EscapeArgument(b"abc0134").to_string(), "abc0134");
    assert_eq!(EscapeArgument(b"abc/0134..").to_string(), "abc/0134..");
    assert_eq!(EscapeArgument(b"abc 0134").to_string(), "'abc 0134'");
    assert_eq!(EscapeArgument(b"abc '134").to_string(), "'abc \\'134'");
    assert_eq!(
        EscapeArgument("α β".as_bytes()).to_string(),
        r#"'\xce\xb1 \xce\xb2'"#
    );
}

#[test]
fn render_labels() {
    let mut output = vec![];
    super::labels(
        "%(pid) - %(pi%(maxrss) - %(time:%F %X) - \\u{221e}",
        &mut output,
    )
    .unwrap();
    assert_eq!(
        std::str::from_utf8(&output),
        Ok("PID - %(piMAX. RSS - TIME - \u{221e}")
    );
}
