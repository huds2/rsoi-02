#!/usr/bin/env bash
set -e

export SCRIPT_PATH=/docker-entrypoint-initdb.d/
export PGPASSWORD=test
psql -f "$SCRIPT_PATH/scripts/fill-bonuses.sql" -d privileges -U program
psql -f "$SCRIPT_PATH/scripts/fill-flights.sql" -d flights -U program
