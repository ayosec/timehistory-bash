# Test to use the shell session after removing the builtin.

load_builtin

/bin/true 1
/bin/true 2

timehistory -s header=true -s table=true

ASSERT_OUTPUT \
  "timehistory -f '%n\t%C'" \
  <<-ITEMS
	NUMBER  COMMAND
	1       /bin/true 1
	2       /bin/true 2
ITEMS
