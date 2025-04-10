#!/bin/bash
set -e 

yarn node:local &
# node can crash if it recieves a message too early
sleep 5
yarn test:local
test_status=$?

yarn node:local:stop
exit $test_status