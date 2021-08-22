# Test for the clear history option.

load_builtin

timehistory -s format='%n'

/bin/true
test -n "$(timehistory)"

timehistory -R
test -z "$(timehistory)"
