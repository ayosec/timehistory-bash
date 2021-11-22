# timehistory for bash

timehistory is a [*loadable builtin*] for bash that tracks programs executed in
the running shell. When a program is finished, timehistory collects the
[resources used] by it. The data can be displayed in a [customizable format] or
as JSON.

[*loadable builtin*]: #what-is-a-loadable-builtin:
[customizable format]: ./FORMAT.md
[resources used]: https://man7.org/linux/man-pages/man2/getrusage.2.html

## Example

```console
$ enable -f /…/libtimehistory_bash.so timehistory

$ head -c 10M /dev/zero | sha512sum
…

$ timehistory
NUMBER  TIME      %CPU  ELAPSED  COMMAND
1       09:29:08  100%  0.046    sha512sum
2       09:29:08  16%   0.045    head -c 10M /dev/zero

$ timehistory -f '[header,table]%n\t%(pid)\t%e\t%P\t%M\t%w\t%C'
NUMBER  PID   ELAPSED  %CPU  MAXRSS  VCSW  COMMAND
1       6651  0.046    100%  4428    2     sha512sum
2       6650  0.045    16%   4488    320   head -c 10M /dev/zero

$ timehistory -v 1
PID:                          6651
Command:                      sha512sum
Time:                         2021-08-22 09:29:08
User time:                    0.042 seconds
System time:                  0.004 seconds
Percent of CPU:               100%
Elapsed time:                 0:00.046
Maximum resident set size:    4428 Kib
Major page faults:            0
Minor page faults:            135
Voluntary context switches:   2
Involuntary context switches: 1
File system inputs:           0
File system outputs:          0
Exit status:                  0
```

## What is a *Loadable Builtin*

Bash, like many other shells, provides [*builtins*]. A builtin is a command
implemented by the shell itself, so it does not need to invoke another program
when the command is executed. Some builtins are used to interact with the shell
(like `cd` or `jobs`), and others are common utilities (like `printf` or
`test`).

A *loadable builtin* is a builtin implemented in a [dynamic library]. Bash can
load a shared object (a `.so` file), and create a new builtin from it.

For example, if the builtin `foo` is implemented in a `libfoo.so` file, it can
be loaded in a running shell with the following command:

```console
$ enable -f /…/libfoo.so foo

$ foo
executes a function from libfoo.so
```

Documentation on *loadable builtins* is very scarce. There are some notes and
examples in the [`examples/loadables`] directory of the bash source code, and a
few articles around the web, but barely anything else.

[*builtins*]: https://www.gnu.org/software/bash/manual/html_node/Shell-Builtin-Commands.html
[`examples/loadables`]: https://git.savannah.gnu.org/cgit/bash.git/tree/examples/loadables?h=bash-5.1
[dynamic library]: https://en.wikipedia.org/wiki/Shared_libraries

## Format Strings

The *format string* controls how to render every entry in the history list.

The syntax for the format string is composed by plain text and *resource
specifiers*. Each *resource specifier* is preceded by a percent sign (`%`).

Some specifiers have an alias. For example, both `%M` and `%(maxrss)` refers to
the maximum resident set size.

Many specifiers are taken from [GNU time], so most format strings for it should
be compatible with timehistory.

To see more details about the syntax, please see [`FORMAT.md`](./FORMAT.md).

[GNU time]: https://www.gnu.org/software/time/

## Installation

timehistory only requires the `.so` built from the sources in this repository.
You can download a [precompiled package](#precompiled-packages), or
[build it from sources](#installation-from-sources).

The `.so` file can be in any directory of the file system. The full path is
required to enable it:

```console
$ enable -f /usr/lib/bash/libtimehistory_bash.so timehistory
```

The directory can be omitted if it is added to the `$BASH_LOADABLES_PATH`
variable:

```console
$ BASH_LOADABLES_PATH=/usr/lib/bash

$ enable -f libtimehistory_bash.so timehistory
```

Currently, timehistory has been tested only in Linux x86-64, but it may work in
other platforms.

### Precompiled Packages

There are packages for some Linux distributions in the [Releases] page. After
installing any of them, timehistory is available in `/usr/lib/bash`.

Alternatively, there is a tarball (built in Debian stable) with the shared
object. To install it, just copy the `libtimehistory_bash.so` file to any path
of the file system.

```console
$ tar xzf timehistory-bash-0.1.0.tar.gz

$ sudo install -t /usr/lib/bash -D -o root -g root libtimehistory_bash.so
```

These packages are built in a GitHub Actions runner.

### Build from Sources

The [Rust compiler] is required to build the package from sources. Once it is
installed, type the following command to generate the shared object:

```console
$ cargo build --release
```

The shared object will be available in `target/release/libtimehistory_bash.so`.
Type the following command to install it in `/usr/lib/bash`:

```console
$ sudo install -t /usr/lib/bash -D -o root -g root target/release/libtimehistory_bash.so
```

[Releases]: https://github.com/ayosec/timehistory-bash/releases
[Rust compiler]: https://www.rust-lang.org/tools/install

## Usage

<table>
<tr>
<td>:information_source:</td>
<td>
This section assumes that the <code>libtimehistory_bash.so</code> file is
installed in the <code>/usr/lib/bash</code> directory.
<br><br>
You must modify the commands to use the actual path if the file is in another
directory.
</td>
</tr>
</table>

timehistory is enabled with the [`enable -f`] command:

```bash
enable -f /usr/lib/bash/libtimehistory_bash.so timehistory
```

Once it is activated, it collects resources for every executed program.

It can be removed from the running shell with `enable -d timehistory`.

### Enable timehistory Automatically

To enable timehistory automatically, add the `enable` command to the
`~/.bashrc` file. However, this method is not recommended because timehistory is
very young, and it still may have bugs that can crash the bash process.

A safer approach is to load it only when you need to collect data. For example,
with a function like this in the `~/.bashrc` file:

```bash
_load_timehistory() {
    enable -f /usr/lib/bash/libtimehistory_bash.so timehistory
    # TIMEHISTORY_LIMIT=…
    # TIMEHISTORY_FORMAT='…'
    # TIMEHISTORY_CMDLINE_LIMIT=…
}
```

Then, type `_load_timehistory` before running the commands that have to be
tracked.

Finally, if you have a separated bash configuration for development environments
(for example, in a Docker container) it can be *not-so-risky* to enable it
automatically.

[`enable -f`]: https://www.gnu.org/software/bash/manual/html_node/Bash-Builtins.html#index-enable

### Display Data

Type `timehistory` to see all entries in the history list.

Every entry is rendered using the default [format string]. To use a different
[format string] you can use the `$TIMEHISTORY_FORMAT` shell variable, or add
the `-f '…'` option in the command-line.

To see a single entry, type `timehistory <n>`, where `<n>` is the number of the
entry. If the number starts with a plus sign (`+`), the number is relative to
the end of the list (`+1` is the last entry, `+2` the previous one, etc.).

Use the `-v` to print entries in an [extended format], similar to `time -v` from
[GNU time].

Use `-j` to print entries in JSON format.

See the [Example](#example) section to see examples of these options.

[extended format]: ./src/format/verbose.fmt

### Track Commands in Shell Scripts

timehistory can be used to collect executed commands in a bash script. The JSON
output allows to get the raw data, so it can be analyzed later.

With the following snippet at the beginning of the script, timehistory will
write a JSON array with data collected from the programs executed by the script
to a `commands.<PID>.json` file.

```bash
#!/bin/bash

enable -f /usr/lib/bash/libtimehistory_bash.so timehistory
trap 'timehistory -j > commands.$$.json' EXIT

# rest of the script
```

### Delete Data

Use the `-R` option to delete all history entries.

### Available Options

Type `timehistory --help` or `help timehistory` to see all available options:

```console
$ timehistory --help
timehistory: timehistory [-f FMT | -v | -j] [<n> | +<n>] | -s | -R
    Displays information about the resources used by programs executed in
    the running shell.

    Options:
      -f FMT    Use FMT as the format string for every history entry,
                instead of the default value.
      -v        Use the verbose format, similar to GNU time.
      -j        Print information as JSON format.
      -s        Print the current configuration settings.
      -R        Remove all entries in the history.

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
```

## Configuration

timehistory configuration can be modified using shell variables:

* `TIMEHISTORY_FORMAT`

    Set the default [format string] for history entries.

    This value is used when the timehistory is invoked without the `-f` option.

* `TIMEHISTORY_LIMIT`

    Set the maximum number of entries stored in the history list.

    When an entry is added to the history, and the number of entries exceeds
    this limit, the oldest entry is removed.

* `TIMEHISTORY_CMDLINE_LIMIT`

    Set the maximum number of bytes from the command line to be added to the
    history.

    If a command line exceeds this limit, then it is truncated.

The current configuration settings are printed with `timehistory -s`:

```console
$ timehistory -s
TIMEHISTORY_FORMAT        = [header,table]%n\t%(time:%X)\t%P\t%e\t%C
TIMEHISTORY_LIMIT         = 500
TIMEHISTORY_CMDLINE_LIMIT = 512
```

[format string]: ./FORMAT.md
