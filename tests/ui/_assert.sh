#! bash
#
# assert functions to be used in *.sh tests.

ASSERT_OUTPUT() {
  local cmd="$1"
  local expected="${2:-}";

  # Read expected output from std if there is only one argument.
  if [ $# -eq 1 ]
  then
    while read -sr
    do
      if [ -n "$expected" ]
      then
        expected+=$'\n'
      fi

      expected+="$REPLY"
    done
  fi

  local output=$(eval "$cmd" 2>&1)

  if [ "$output" != "$expected" ]
  then
    printf 'assertion failed: invalid output for "%s"\n' "$cmd"
    printf ' expected: %q\n' "$expected"
    printf '   output: %q\n' "$output"

    exit 1
  fi

  return 0
}

ASSERT_FAILS() {
  if eval "$*"
  then
    printf 'assertion failed: "%s" should fail\n' "$*"
    exit 1
  fi

  return 0
}
