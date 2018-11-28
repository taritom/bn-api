#!/usr/bin/env bash

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"

TRAVIS_ENV="${SCRIPT_DIR}/../.travis/.env"
if [[ ! -f "$TRAVIS_ENV" ]]; then
    echo ".travis/.env not found"
    exit 1
fi

type travis &> /dev/null
if [[ "$?" != 0 ]]; then
    echo "travis CLI not found. Please run 'gem install travis'"
    exit 1
fi

echo "Please wait. Encrypting variables one at a time..."
cat $TRAVIS_ENV | xargs -I{} travis encrypt -r big-neon/bn-api {} --add env.global

