load_builtin

timehistory -L 5
for N in {1..10}
do
  /bin/true $N
done

timehistory -f '%n\t%C'
