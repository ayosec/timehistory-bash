# Test to print a single entry from the history.

load_builtin

/bin/true 1
/bin/true 2
/bin/true 3

ASSERT_OUTPUT \
  "timehistory -f '%n %C' 2" \
  "2 /bin/true 2"

ASSERT_OUTPUT \
  "timehistory -f '%n %C' +1" \
  "3 /bin/true 3"

( timehistory x 2>&1 || : ) \
  | grep 'timehistory: invalid digit found in string'
