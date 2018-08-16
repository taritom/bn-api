#!/bin/bash

# This hook will
# - ensure that rustfmt is installed
# - run rustfmt
#
# To enable it, copy this file as .git/hooks/pre-commit (drop the `.sh` extension)

result=0

rustfmt --version &>/dev/null
if [ $? != 0 ]; then
    printf "[pre_commit] \033[0;31merror\033[0m: \"rustfmt\" not available?\n"
    result=1
fi

if [[ $result != 0 ]]; then
    printf "[pre_commit] rustfmt is not installed"
    exit 1
fi

problem_files=()

printf "[pre_commit] rustfmt "
for file in $(git diff --name-only --cached --diff-filter=MAR); do
    if [ ${file: -3} == ".rs" ]; then
        rustfmt --skip-children --write-mode=diff $file &>/dev/null
        if [ $? != 0 ]; then
        	problem_files+=($file)
            result=1
        fi
    fi
done

if [ $result != 0 ]; then
    printf "\033[0;31mfail\033[0m \n"
    printf "[pre_commit] the following files need formatting: \n"

    for file in $problem_files; do
        printf "    rustfmt $file\n"
        rustfmt $file
    done
else
  printf "\033[0;32mok\033[0m \n"
fi

exit $result