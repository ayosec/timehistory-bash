# Formatting Syntax

The format string controls how to show every entry in the history. It consists
of resource specifiers (described below) and plain text.

It is based on the [GNU time], and most format strings for [GNU time] should be
usable here.

## Specifiers

The following resource specifiers are accepted in the format string:

| Specifiers | Description |
|------------|-------------|
|`\\` | A backslash. |
|`\e` | An ESC character. |
|`\n` | A newline. |
|`\t` | A tab character. |
|`\u{H*}` | A Unicode character. |
|`%%` | A literal '%'. |
|`%(pid)` | Process identifier. |
|`%(sys_time_us)` | System (kernel) time (microseconds). |
|`%(time:FORMAT)` | Start time with a custom format. |
|`%(user_time_us)` | User time (microseconds). |
|`%C`<br>`%(args)` | Command name and arguments. |
|`%c`<br>`%(nivcsw)` | Involuntary context switches. |
|`%E` | Elapsed real (wall clock) time in [hour:]min:sec. |
|`%e` | Elapsed real time in seconds. |
|`%F`<br>`%(majflt)` | Major page faults (required physical I/O). |
|`%I`<br>`%(inblock)` | File system inputs. |
|`%M`<br>`%(maxrss)` | Maximum resident set size in Kib. |
|`%n` | Entry number in the history. |
|`%O`<br>`%(oublock)` | File system outputs. |
|`%P`<br>`%(cpu)` | Percent of CPU this job got. |
|`%R`<br>`%(minflt)` | Minor page faults (reclaims; no physical I/O involved). |
|`%S`<br>`%(sys_time)` | System (kernel) time (seconds). |
|`%Tn` | Signal number, if terminated by a signal. |
|`%Tt` | Termination type: normal, signalled, stopped. |
|`%Tx` | Exit code, if terminated normally. |
|`%u` | Elapsed real time in microseconds. |
|`%U`<br>`%(user_time)` | User time (seconds). |
|`%w`<br>`%(nvcsw)` | Voluntary context switches. |
|`%x`<br>`%(status)` | Exit status of command. |

## Options

Options are surrounded by brackets at the beginning of the format string. There
are two valid options:

* `header`

    Print a header containing the field labels.

* `table`

    Render the history list as a table. Columns are separated by the tab
    character.

Example:

    [header,table]%n\t%e\t%C

## Date/Time Format

The syntax for the `%(time)` specifier is from the [chrono library].

Examples:

    %(time:%F %X)     YYYY-MM-DD hh:mm:ss
    %(time:%+)        ISO-8601.
    %(time:%s)        UNIX timestamp.

[chrono library]: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
[GNU time]: https://www.gnu.org/software/time/
