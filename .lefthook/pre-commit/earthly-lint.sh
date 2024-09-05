#!/bin/bash
command -v "earthly" >/dev/null 2>&1 ||
	{
		echo >&2 "I require earthly but it's not installed.  Aborting."
		exit 1
	}

FILES=$(git diff --name-only --cached | grep '^Earthfile$')

ERRORS=0
for file in $FILES; do
	if [ ! -f "$file" ]; then
		echo >&2 "ERROR: $file does not exist"
		exit 1
	fi
	foldername=$(dirname "$file")
	if ! cd "$foldername"; then
		echo "cd failed for $foldername"
		exit 1
	fi
	foldername=$(dirname "$file")
	if ! cd "$foldername"; then
		echo "cd failed for $foldername"
		exit 1
	fi
	RESULT=$(earthly ls 2>&1 >/dev/null)
	# shellcheck disable=SC2181
	if [ $? -gt 0 ]; then
		echo "$file - $RESULT"
		exit 1
	else
		echo "$file is valid"
	fi
done
if [ $ERRORS -gt 0 ]; then
	exit 1
fi
exit 0
