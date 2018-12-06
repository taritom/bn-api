#!/usr/bin/env bash

if [[ -z "$CI" ]]; then
    echo "Script should be run in travis only"
    exit 1
fi

echo "+git checkout master"
git checkout master
echo "+./scripts/init-github-ssh.sh"
./scripts/init-github-ssh.sh "$encrypted_319ed1854cd7_key" "$encrypted_319ed1854cd7_iv"
echo "+./scripts/bump-version.sh --with-git"
./scripts/bump-version.sh --with-git
