# Test to print a single entry from the history.

load_builtin

ASSERT_OUTPUT \
  "timehistory -f '[header]%n %C'" \
  "NUMBER COMMAND"
