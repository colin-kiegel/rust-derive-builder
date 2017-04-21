#!/bin/bash

# based on
#       - https://gist.github.com/zofrex/4a5084c49e4aadd0a3fa0edda14b1fa8
#       - https://gist.github.com/chadmaughan/5889802
#
# to install it
#       links this to '.git/hooks/pre-push' via
#       `(cd .git/hooks && ln -s ../../dev/githook.sh pre-push)`
#
# to skip the tests, run with the --no-verify argument
# 			e.g. - $ 'git push --no-verify'
#
# configure via
#       git config hooks.usecolor true
#       git config hooks.rustup true
#       git config hooks.checkformat true
#       git config hooks.checkstable true
#       git config hooks.checkbeta true
#       git config hooks.checknightly true
#       git config hooks.nightlytests true
#       git config hooks.checkfeatures true
#
# Note this will stash all local changes.
set -u

errors=0

function main {
	load_config

	check_clean_state

	[ "$checkformat" == true ] && check_format
	[ "$checkstable" == true ] && run_tests_on "stable"
	[ "$checkbeta" == true ] && run_tests_on "beta"
	[ "$checknightly" == true ] && run_tests_on "nightly"
	[ "$nightlytests" == true ] && run_script "dev/nightlytests.sh"
	[ "$checkfeatures" == true ] && run_script "dev/checkfeatures.sh"

	if [ "$errors" != 0 ]; then
		echo -e "${FMT_ERR}EE${FMT_RESET}: Some checks failed!"
		exit 1
	else
		echo -e "${FMT_OK}OK${FMT_RESET}: All checks passed!"
	fi
}

function check_format {
	echo_begin "Checking formatting"
	diff=$(
	  exec 2>&1
		rustup run stable cargo fmt -- --write-mode diff
	)
	stripped_diff=$(echo "$diff" | sed -e '/^Diff of/d' -e '/^$/d')
  test -z "$stripped_diff"
	check_or_echo $? "" "$diff"
}

function run_tests_on {
	echo_begin "Running tests on $1"
	result=$(
	  exec 2>&1
		if [ "$rustup" == true ]; then
			rustup update $1 || exit
		fi
		cd derive_builder && rustup run "$1" cargo test --all --color always --features skeptic_tests
	); ret=$?
	check_or_echo $ret "" "$result"
}

function run_script {
	echo_begin "Running $1"
	result=$(
	  exec 2>&1
		($1)
	); ret=$?
	check_or_echo $ret "" "$result"
}

function check_clean_state {
	echo_begin "Checking clean working directory"

	result=$(git status --porcelain)
	if [[ -z "$result" ]]; then
		check_or_echo 0 "" "$result"
	else
		check_or_echo 1 "please commit or stash changes!" "$result"
	fi
}

function load_config {
	set_color false
	config_status=0
	lookup_git_flag usecolor
	set_color "$usecolor"
	lookup_git_flag rustup
	lookup_git_flag checkformat
	lookup_git_flag checkstable
	lookup_git_flag checkbeta
	lookup_git_flag checknightly
	lookup_git_flag nightlytests
	lookup_git_flag checkfeatures

  if [ $config_status -ne 0 ]; then
		echo -e "${FMT_ERR}EE${FMT_RESET}: Invalid git configuration. Aborting checks."
		exit 1
	fi
}

function set_color {
	if [ "$1" == true ]; then
	  OK_SIGN='\xE2\x9C\x93' # check char
	  ERR_SIGN='\xE2\x9D\x8C' # cross char
		FMT_OK='\033[1;32m' # bold green
	  FMT_ERR='\033[1;31m' # bold red
	  FMT_INFO='\033[1;33m' # bold yellow
	  FMT_RESET='\033[0m' # reset

  else
		OK_SIGN='(OK)'
	  ERR_SIGN='(EE)'
	  FMT_OK=''
	  FMT_ERR=''
	  FMT_INFO=''
	  FMT_RESET=''
	fi
}

function lookup_git_flag {
	# lookup the boolean value of 'hooks.$1' (i.e. true or false)
	local flag="$(git config --bool hooks.$1)"
	if [ "$flag" == true ] || [ "$flag" == false ]; then
		# echo -e "${FMT_INFO}II${FMT_RESET}: found $1=$flag"
		eval "${1}='${flag}'"
	else
		>&2 echo -e "${FMT_ERR}EE${FMT_RESET}: git flag missing: hooks.$1"
		>&2 echo "    you can set it like"
		>&2 echo "  $ git config hooks.$1 true # (or false)"
		>&2 echo

		config_status=1
	fi
}

function echo_begin {
	# suppress line-feeds, so next echo will be on the same line
	echo -en "${FMT_INFO}II${FMT_RESET}: "
	echo -n "$*... "
}

function check_or_echo {
	# $1 expected to be an exit code, e.g. $? for the last command
	# $2 status message
	# $3 expected to be the captured stdout of the program
	if [ $1 -eq 0 ]; then
		echo -e "${FMT_OK}${OK_SIGN} ${2:-}${FMT_RESET}"
	else
		echo -e "${FMT_ERR}${ERR_SIGN} ${2:-}${FMT_RESET}"
		[ $# -ge 3 ] && printf '%s\n' "${@:3}" && echo
		errors+=1
	fi
}

function base_dir {
  if [ $(uname -s) != "Darwin" ] && hash readlink 2>/dev/null; then
    # use readlink, if installed, to follow symlinks
    local __DIR="$(dirname "$(readlink -f "$0")")"
  else
    local __DIR="$(dirname "$0")"
  fi
  echo ${__DIR}
}

# cd into the crate root
(cd "$(base_dir)/.." && main $@)
