load_builtin

/bin/true 1
/bin/true 2
/bin/true 3

timehistory -f '%n %C' 2
timehistory -f '%n %C' +1

timehistory x || :
