#!/usr/bin/env bash
cd $(dirname "$0")
pwd
sqlite3 ../toad.db < db.sql
