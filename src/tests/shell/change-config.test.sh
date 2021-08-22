# bash
# Test to change builtin configuration.

load_builtin

timehistory -s limit=5000 -s format='%n\t%P\t%C'

ASSERT_OUTPUT \
  "timehistory -s" \
  <<-'ITEMS'
	format = %n\t%P\t%C
	limit  = 5000
ITEMS

timehistory -s format='> %C'

command expr 1 + 2
ASSERT_OUTPUT \
  "timehistory" \
  "> expr 1 '+' 2"
