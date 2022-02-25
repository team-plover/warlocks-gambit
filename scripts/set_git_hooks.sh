#!/bin/bash

if [ ! -e .git ]; then
	echo "Must be run from repository root"
	exit 1
fi

for src in scripts/*.git-hook; do
	dest=${src%.git-hook}
	dest=${dest#scripts/}
	cp -f $src .git/hooks/$dest
	echo "Copied $dest hook"
done
