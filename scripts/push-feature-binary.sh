#!/usr/bin/env bash


BINARIES_DIR=binaries
BINARIES=(server bndb_cli api-cli)
BRANCH="$DRONE_SOURCE_BRANCH"
REPO=git@github.com:Krakaw/bn-api-releases.git
REPO_DIR=bn-api-releases
BASE=$(pwd)

mkdir -p "$HOME/.ssh/"
cd "$HOME"

ssh-keyscan github.com >~/.ssh/known_hosts
eval $(ssh-agent -s)
ssh-add <(echo "$GITHUB_RELEASE_SSH_KEY")
git config --global user.email "$GH_USER_EMAIL"
git config --global user.name "$GH_USER_NAME"
git config --global core.sparseCheckout true

git clone "$REPO"
cd "$REPO_DIR"
git checkout "$BRANCH" || git checkout -b "$BRANCH"
[ ! -d "$BINARIES_DIR" ] && mkdir "$BINARIES_DIR"

for BINARY in "${BINARIES[@]}"
do
  echo "Copying ${BASE}/target/release/$BINARY -> $BINARIES_DIR/$BINARY"
  cp "${BASE}/target/release/$BINARY" "$BINARIES_DIR/$BINARY"
  git add "$BINARIES_DIR/$BINARY"
done

git commit -a -m "Commit binaries for $BRANCH" || exit 0
git push -u origin "$BRANCH"
