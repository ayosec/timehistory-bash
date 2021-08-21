# bash
# Test to change builtin configuration.

load_builtin

timehistory -L 5000 -F '%n\t%P\t%C'

ASSERT_OUTPUT \
  "timehistory -C" \
  $'-L 5000 -F \'%n\\\\t%P\\\\t%C\''

timehistory -F '> %C'

command expr 1 + 2
ASSERT_OUTPUT \
  "timehistory" \
  "> expr 1 '+' 2"
