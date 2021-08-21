# Test to use the shell session after a panic in the builtin.

load_builtin

type timehistory

/bin/true
ASSERT_OUTPUT \
  "timehistory -f '- %n'" \
  "- 1"

# Force a panic.
ASSERT_FAILS timehistory -P
ASSERT_FAILS timehistory

/bin/echo 2
enable -d timehistory
ASSERT_FAILS type timehistory

ASSERT_OUTPUT \
  "/bin/echo 3" \
  "3"
