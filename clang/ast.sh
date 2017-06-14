#!/bin/bash

set -e

CFLAGS="-g -Wall -Werror"
CFILES="./test.c"

mkdir -p ./ast
i=0
for cfile in ${CFILES}; do
    target=./ast/$(basename $cfile).ast
    clang ${CFLAGS} -S -emit-ast $cfile -o $target
    echo $cfile to $target

    i=$((i + 1))
done
echo "found $i files"
