# Formatting Syntax

The format string controls how to show every entry in the history. It consists
of resource specifiers (described below) and plain text.

It is based on the [GNU time], and most format strings for [GNU time] should be
usable here.

## Specifiers

The following resource specifiers are accepted in the format string:

| Specifiers | Description |
|------------|-------------|
|`%%` | A literal '%'. |
|`\n` | A newline. |
|`\t` | A tab character. |
|`\e` | An ESC character. |
|`\\` | A backslash. |
|`\u{H*}` | A Unicode character. |
|`%C`<br>`%(args)` | Command name and arguments. |
|`%E` | Elapsed real (wall clock) time in [hour:]min:sec. |
|`%F`<br>`%(majflt)` | Major page faults (required physical I/O). |
|`%I`<br>`%(inblock)` | File system inputs. |
|`%M`<br>`%(maxrss)` | Maximum resident set size in Kib. |
|`%O`<br>`%(oublock)` | File system outputs. |
|`%P`<br>`%(cpu)` | Percent of CPU this job got. |
|`%R`<br>`%(minflt)` | Minor page faults (reclaims; no physical I/O involved). |
|`%S`<br>`%(sys_time)` | System (kernel) time (seconds). |
|`%(sys_time_us)` | System (kernel) time (microseconds). |
|`%Tt` | Termination type: normal, signalled, stopped. |
|`%Tn` | Signal number, if terminated by a signal. |
|`%Tx` | Exit code, if terminated normally. |
|`%U`<br>`%(user_time)` | User time (seconds). |
|`%(user_time_us)` | User time (microseconds). |
|`%W`<br>`%(nswap)` | Times swapped out. |
|`%X`<br>`%(ixrss)` | Average amount of shared text in Kib. |
|`%Z`<br>`%(page_size)` | Page size. |
|`%c`<br>`%(nivcsw)` | Involuntary context switches. |
|`%e` | Elapsed real time in seconds. |
|`%k`<br>`%(nsignals)` | Signals delivered. |
|`%n` | Unique identifier in the history. |
|`%p`<br>`%(isrss)` | Average unshared stack size in Kib. |
|`%r`<br>`%(msgrcv)` | Socket messages received. |
|`%s`<br>`%(msgsnd)` | Socket messages sent. |
|`%t`<br>`%(idrss)` | Average resident set size in Kib. |
|`%u` | Elapsed real time in microseconds. |
|`%w`<br>`%(nvcsw)` | Voluntary context switches. |
|`%x`<br>`%(status)` | Exit status of command. |
|`%(pid)` | Process identifier. |
|`%(time:FORMAT)` | Start time with a custom format. |

## Date/Time Format

The syntax for the `%(time)` specifier is from the [chrono library].

Examples:

    %(time:%F %X)     YYYY-MM-DD hh:mm:ss
    %(time:%+)        ISO-8601.
    %(time:%s)        UNIX timestamp.

[chrono library]: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
[GNU time]: https://www.gnu.org/software/time/
