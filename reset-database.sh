#!/usr/bin/env bash
echo Dropping database..
bndb_cli drop -c $DATABASE_URL
echo Recreating database..
bndb_cli create -c $DATABASE_URL -e $SUPERUSER_EMAIL -m $SUPERUSER_MOBILE -p $SUPERUSER_PASSWORD