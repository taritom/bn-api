#!/usr/bin/env bash

set -e

if [[ -z "$CI" ]]; then
    echo "Script should be run in CI only"
    exit 1
fi

if [[ -z "$GITHUB_SSH_KEY" ]]; then
    echo "GITHUB_SSH_KEY not set. Exiting"
    exit 1
fi

mkdir -p $HOME/.ssh/

ssh-keyscan github.com > ~/.ssh/known_hosts 2> /dev/null
eval $(ssh-agent -s)
ssh-add <(echo "$GITHUB_SSH_KEY")

git config --global user.email "$GH_USER_EMAIL"
git config --global user.name "$GH_USER_NAME"

git checkout master

git remote add sshremote git@github.com:$DRONE_REPO.git


