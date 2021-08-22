//! Extract options from a format string.

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct FormatOptions<'a> {
    pub header: bool,
    pub table: bool,
    pub format: &'a str,
}

impl FormatOptions<'_> {
    pub fn parse(mut format: &str) -> FormatOptions {
        let mut header = false;
        let mut table = false;

        if format.starts_with('[') {
            if let Some(end) = format.find(']') {
                let (options, fmt) = format[1..].split_at(end - 1);
                format = &fmt[1..];

                for option in options.split(',') {
                    match option {
                        "header" => header = true,
                        "table" => table = true,
                        o => bash_builtins::warning!("'{}': invalid format option.", o),
                    }
                }
            }
        }

        FormatOptions {
            header,
            table,
            format,
        }
    }
}

#[test]
fn parse_options() {
    assert_eq!(
        FormatOptions::parse("abc"),
        FormatOptions {
            header: false,
            table: false,
            format: "abc"
        }
    );

    assert_eq!(
        FormatOptions::parse("[header]abc"),
        FormatOptions {
            header: true,
            table: false,
            format: "abc"
        }
    );

    assert_eq!(
        FormatOptions::parse("[table,header]abc"),
        FormatOptions {
            header: true,
            table: true,
            format: "abc"
        }
    );

    assert_eq!(
        FormatOptions::parse("[]abc"),
        FormatOptions {
            header: false,
            table: false,
            format: "abc"
        }
    );
}

#[cfg(test)]
mod mock_bash_fns {
    #[no_mangle]
    extern "C" fn builtin_warning(_: *const libc::c_char) {}
}
