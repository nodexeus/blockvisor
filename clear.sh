NODES=$(bv node list | tail -n +3 | head -n -2 | awk '{print $2}' | sed 's/\x1b\[[^\x1b]*m//g')

for node in $NODES
do
  bv node delete ${node}
done
