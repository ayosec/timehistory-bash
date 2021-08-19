load_builtin

/bin/echo 1
timehistory -f '- %n'

enable -d timehistory
/bin/echo 2

load_builtin
/bin/echo 3
timehistory -f '- %n'
