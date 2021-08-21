# Test to track commands in a subshell.

load_builtin

timehistory -F '%C'
(
  /bin/echo 1
  /bin/echo 2
  /bin/false
) &

wait
ASSERT_OUTPUT \
  "timehistory -f '%Tx,%C'" \
  <<-ITEMS
	0,/bin/echo 1
	0,/bin/echo 2
	1,/bin/false
ITEMS
