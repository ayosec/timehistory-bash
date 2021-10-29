# Changelog

## 0.2.1 - 2021-10-29

* Limit how many bytes are copied from the command line with the `TIMEHISTORY_CMDLINE_LIMIT` variable.

## 0.2.0 - 2021-08-30

Configuration is now set with shell variables:

* `TIMEHISTORY_FORMAT` sets the default format string (when `-f` is not given).
* `TIMEHISTORY_LIMIT` sets the maximum history size.

The old system (`-s option=value`) is still available but deprecated. It is hidden from the documentation.
