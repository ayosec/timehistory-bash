# bash
# Test to change builtin configuration.

load_builtin

TIMEHISTORY_LIMIT=5000
TIMEHISTORY_FORMAT='%n\t%P\t%C'
TIMEHISTORY_CMDLINE_LIMIT=1000

ASSERT_OUTPUT \
  "timehistory -s" \
  <<-'ITEMS'
	TIMEHISTORY_FORMAT        = %n\t%P\t%C
	TIMEHISTORY_LIMIT         = 5000
	TIMEHISTORY_CMDLINE_LIMIT = 1000
ITEMS

timehistory -s format='> %C'

command expr 1 + 2
ASSERT_OUTPUT \
  "timehistory" \
  "> expr 1 '+' 2"


# Backward compatibility.
timehistory -s limit=123 -s format='%N\t%P'
ASSERT_OUTPUT \
  'echo "${TIMEHISTORY_FORMAT:-NA} ${TIMEHISTORY_LIMIT:-NA}"' \
  '%N\t%P 123'


# Check variables after deleting the builtin.
enable -d timehistory
ASSERT_OUTPUT \
  'echo "${TIMEHISTORY_FORMAT:-NA} ${TIMEHISTORY_LIMIT:-NA}"' \
  '%N\t%P NA'
