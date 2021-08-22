# Test to use the shell session after removing the builtin.

load_builtin

/bin/true 1
/bin/true 2

ASSERT_OUTPUT \
  "timehistory -f '[header,table]%n\t%C'" \
  <<-ITEMS
	NUMBER  COMMAND
	1       /bin/true 1
	2       /bin/true 2
ITEMS

ASSERT_OUTPUT \
  "timehistory -f '[table]%n\t%C'" \
  <<-ITEMS
	1  /bin/true 1
	2  /bin/true 2
ITEMS
