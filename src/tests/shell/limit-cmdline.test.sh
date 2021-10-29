# Test to print a single entry from the history.

load_builtin

TIMEHISTORY_CMDLINE_LIMIT=50

/bin/true {1..10}
/bin/true {1000..1100}

ASSERT_OUTPUT \
  "timehistory -f '%n %C'" \
  <<-ITEMS
	1 /bin/true 1 2 3 4 5 6 7 8 9 10
	2 /bin/true 1000 1001 1002 1003 1004 1005 1006 1007 1008 1009 1
ITEMS
