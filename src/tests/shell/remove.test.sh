# Test to use the shell session after removing the builtin.

load_builtin

/bin/echo 1
ASSERT_OUTPUT \
  "timehistory -f '- %n'" \
  "- 1"

enable -d timehistory
/bin/echo 2

ASSERT_FAILS timehistory

load_builtin
/bin/echo 3

ASSERT_OUTPUT \
  "timehistory -f '> %n'" \
  "> 1"
