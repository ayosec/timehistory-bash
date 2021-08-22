# Test for the history limit.

load_builtin

timehistory -s limit=5
for N in {1..10}
do
  /bin/true $N
done

ASSERT_OUTPUT \
  "timehistory -f '%n,%C'" \
  <<-ITEMS
	6,/bin/true 6
	7,/bin/true 7
	8,/bin/true 8
	9,/bin/true 9
	10,/bin/true 10
ITEMS
