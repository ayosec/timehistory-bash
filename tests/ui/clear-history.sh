# Test for the clear history option.

load_builtin

/bin/true
test -n "$(timehistory)"

timehistory -R
test -z "$(timehistory)"
