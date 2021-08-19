load_builtin

type timehistory

/bin/true
timehistory -f '- %n'

# Force a panic.
timehistory -P &> /dev/null
timehistory

/bin/echo 2
enable -d timehistory
type timehistory

/bin/echo 3
