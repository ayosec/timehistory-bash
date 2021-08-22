# Formatting Syntax

The format string controls how to show every entry in the history. It consists
of resource specifiers (described below) and plain text.

It is based on the [GNU time], and most format strings for [GNU time] should be
usable here.

## Specifiers

The following resource specifiers are accepted in the format string:

| Specifiers | Description |
|------------|-------------|
%SPECS%

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
