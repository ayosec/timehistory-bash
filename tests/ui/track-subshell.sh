load_builtin

timehistory -F '%C'
(
  /bin/echo 1
  /bin/echo 2
  /bin/false
) &

wait
timehistory -f '%Tx,%C'
