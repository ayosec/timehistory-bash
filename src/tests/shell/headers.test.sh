# Test to print a single entry from the history.

load_builtin

timehistory -s header=true
ASSERT_OUTPUT \
  "timehistory -f '%n %C'" \
  "NUMBER COMMAND"
