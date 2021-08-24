// This file contains the specifiers for the formatting syntax of this builtin.
//
// Each specifiers if defined by an entry with the following structure:
//
//      : %a %b %c
//          //! [alias] …
//          //! description
//          rust code
//
// The supported specifiers are mostly compatible with GNU time.

// Literal characters.

: %%
    //! A literal '%'.
    w!('%');

: \n
    //! A newline.
    w!('\n');

: \t
    //! A tab character.
    w!('\t');

: \e
    //! An ESC character.
    w!('\x1b');

: \\
    //! A backslash.
    w!('\\');

: \u{
    //! [alias] \u{H*}
    //! A Unicode character.

    let code = &format[chr_index..];
    let right = memchr::memchr(b'}', code);
    let chr = right
        .and_then(|i| std::str::from_utf8(&code[1..i]).ok())
        .and_then(|s| u32::from_str_radix(s, 16).ok())
        .and_then(char::from_u32);

    match (right, chr) {
        (Some(right), Some(chr)) => {
            // Advance the input iterator.
            for _ in 0..right {
                input.next();
            }

            w!(chr);
        },

        _ => {
            discard_spec!();
        }
    }


// Specifiers.

: %N %(filename)
    //! [label] FILENAME
    //! Filename of the executable.
    w!(EscapeArgument(entry.filename.as_bytes()));

: %C %(args)
    //! [label] COMMAND
    //! Command name and arguments.
    let mut need_space = false;
    for arg in entry.args.iter() {
        if mem::replace(&mut need_space, true) {
            w!(" ");
        }

        w!(EscapeArgument(arg.as_bytes()));
    }

: %E
    //! [label] ELAPSED
    //! Elapsed real (wall clock) time in [hour:]min:sec.
    if let State::Finished { running_time: Some(time), .. } = &entry.state {
        let (secs, ms) = (time.as_secs(), time.subsec_millis());
        if secs >= 3660 {
            w!("{}:{:02}:{:02}", secs / 3660, (secs % 3660) / 60, secs % 60);
        } else {
            w!("{}:{:02}.{:03}", secs / 60, secs % 60, ms);
        }
    }

: %F %(majflt)
    //! [label] MAJFL
    //! Major page faults (required physical I/O).
    rusage_field!(ru_majflt);

: %I %(inblock)
    //! [label] FSIN
    //! File system inputs.
    rusage_field!(ru_inblock);

: %M %(maxrss)
    //! [label] MAXRSS
    //! Maximum resident set size in Kib.
    rusage_field!(ru_maxrss);

: %O %(oublock)
    //! [label] FSOUT
    //! File system outputs.
    rusage_field!(ru_oublock);

: %P %(cpu)
    //! [label] %CPU
    //! Percent of CPU this job got.
    if let State::Finished { running_time: Some(time), rusage, .. } = &entry.state {
        // Use milliseconds instead of microseconds to avoid weird values for
        // very-short commands, like `/bin/true`.
        let elapsed = time.as_millis();

        if elapsed > 0 {
            let usage_time =
                rusage.ru_utime.tv_sec * 1_000 + rusage.ru_utime.tv_usec / 1000 +
                rusage.ru_stime.tv_sec * 1_000 + rusage.ru_stime.tv_usec / 1000;

            let pcent = (1000000 * usage_time as u128 / elapsed) as f64 / 10000.0;
            w!("{:.1$}%", pcent, if pcent < 10.0 { 2 } else { 0 });
        } else {
            w!("0%");
        }
    }

: %R %(minflt)
    //! [label]  MINFL
    //! Minor page faults (reclaims; no physical I/O involved).
    rusage_field!(ru_minflt);

: %S %(sys_time)
    //! [label] SYSTIME
    //! System (kernel) time (seconds).
    if let State::Finished { rusage, .. } =  &entry.state {
        let time = &rusage.ru_stime;
        w!("{}.{:03}", time.tv_sec, time.tv_usec / 1000);
    }

: %(sys_time_us)
    //! [label] SYSTIME
    //! System (kernel) time (microseconds).
    if let State::Finished { rusage, .. } =  &entry.state {
        let time = &rusage.ru_stime;
        w!("{}", time.tv_sec * 1_000_000 + time.tv_usec);
    }

: %Tt
    //! [label] EXTYPE
    //! Termination type: normal, signalled, stopped.
    if let State::Finished { status, .. } = &entry.state {
        w!(
            if libc::WIFSTOPPED(*status) { "stopped" }
            else if libc::WIFSIGNALED(*status) { "signalled" }
            else { "normal" }
        );
    }

: %Tn
    //! [label] SIGNAL
    //! Signal number, if terminated by a signal.
    if let State::Finished { status, .. } = &entry.state {
        if libc::WIFSIGNALED(*status) {
            w!(libc::WTERMSIG(*status));
        }
    }

: %Tx
    //! [label] EXIT
    //! Exit code, if terminated normally.
    if let State::Finished { status, .. } = &entry.state {
        if libc::WIFEXITED(*status) {
            w!(libc::WEXITSTATUS(*status));
        }
    }

: %U %(user_time)
    //! [label] USERTIME
    //! User time (seconds).
    if let State::Finished { rusage, .. } =  &entry.state {
        let time = &rusage.ru_utime;
        w!("{}.{:03}", time.tv_sec, time.tv_usec / 1000);
    }

: %(user_time_us)
    //! [label] USERTIME
    //! User time (microseconds).
    if let State::Finished { rusage, .. } =  &entry.state {
        let time = &rusage.ru_utime;
        w!("{}", time.tv_sec * 1_000_000 + time.tv_usec);
    }

: %c %(nivcsw)
    //! [label] IVCSW
    //! Involuntary context switches.
    rusage_field!(ru_nivcsw);

: %e
    //! [label] ELAPSED
    //! Elapsed real time in seconds.
    if let State::Finished { running_time: Some(time), .. } = &entry.state {
        w!("{}.{:03}", time.as_secs(), time.subsec_millis());
    }

: %n
    //! [label] NUMBER
    //! Entry number in the history.
    w!(entry.number);

: %u
    //! [label] ELAPSED
    //! Elapsed real time in microseconds.
    if let State::Finished { running_time: Some(time), .. } = &entry.state {
        w!("{}", time.as_micros());
    }

: %w %(nvcsw)
    //! [label] VCSW
    //! Voluntary context switches.
    rusage_field!(ru_nvcsw);

: %x %(status)
    //! [label] STATUS
    //! Exit status of command.
    if let State::Finished { status, .. } = &entry.state {
        w!(*status);
    }

: %(pid)
    //! [label] PID
    //! Process identifier.
    w!(entry.pid);

: %(time:
    //! [label] STARTED
    //! [label-until] )
    //! [alias] %(time:FORMAT)
    //! Start time with a custom format.

    // Find the right parenthesis to extract the format.
    let timefmt = &format[chr_index..];
    match memchr::memchr(b')', timefmt) {
        None => discard_spec!(),

        Some(right_paren) => {
            // Advance the input iterator.
            for _ in 0..right_paren {
                input.next();
            }

            // `timefmt[1..right_paren]` must always be a valid UTF-8, but we
            // are using `from_utf8_lossy()` to detect possible bugs.
            let fmt = String::from_utf8_lossy(&timefmt[1..right_paren]);
            w!(entry.start_time.format(&fmt));
        }
    }

// vim: ft=rust
