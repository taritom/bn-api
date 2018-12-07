#!/usr/bin/env bash

if [[ -z "$CI" ]]; then
    echo "Script should be run in travis only"
    exit 1
fi

echo "+init-github-ssh.sh"
./scripts/init-github-ssh.sh
echo "+git checkout master"
git checkout master
echo "+git remote add sshremote git@github.com:big-neon/bn-api.git"
git remote add sshremote git@github.com:big-neon/bn-api.git
echo "+bump-version.sh --with-git"
./scripts/bump-version.sh --with-git
