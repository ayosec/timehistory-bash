# Test to print a single entry from the history.

load_builtin

ASSERT_OUTPUT \
  "timehistory -h -f '%n %C'" \
  "NUMBER COMMAND"
