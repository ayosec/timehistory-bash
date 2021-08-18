load_builtin

timehistory -L 5000 -F '%n\t%P\t%C'
timehistory -C

timehistory -F '> %C'

command expr 1 + 2
timehistory
