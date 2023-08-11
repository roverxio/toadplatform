#!/usr/bin/env bash
cd $(dirname "$0")
sqlite3 ../toad.db < db.sql
