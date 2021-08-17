The format string controls how to show every entry in the history. It consists
of resource specifiers (described below) and plain text.

It is based on the GNU time, and most format strings for GNU time should be
usable here.


SPECIFIERS

    The following resource specifiers are accepted in the format string:

%SPECS%


DATE/TIME FORMAT

    The syntax for the %(time) specifier is from the [1]chrono library.

    Examples:

        %(time:%F %X)     YYYY-MM-DD hh:mm:ss
        %(time:%+)        ISO-8601.
        %(time:%s)        UNIX timestamp.

    [1]: https://docs.rs/chrono/latest/chrono/format/strftime/index.html
