#!/bin/bash
#
# - Execute all commands passed to this script
# - Give feedback for each command
# - Accumulate the error code
#
# Examples
#
# - `./.travis-run-all.sh "echo foo; exit 1" "echo bar"`

exit_with=0;

FMT_OK='\033[1;32m' # bold green
FMT_ERR='\033[1;31m' # bold red
FMT_INFO='\033[1;33m' # bold yellow
FMT_RESET='\033[0m' # reset

for cmd in "$@"; do
  echo -e "\n${FMT_INFO}Running \`${cmd}\`${FMT_RESET}"

  (eval $cmd); ret_code=$?

  if [ $ret_code -eq 0 ]; then
    color="$FMT_OK"
  else
    color="$FMT_ERR"
    exit_with=$ret_code
  fi
  echo -e "${color}The command \`${cmd}\` exited with ${ret_code}.${FMT_RESET}"
done

exit $exit_with
