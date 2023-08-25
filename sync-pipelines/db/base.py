from peewee import PostgresqlDatabase

import sqlite3

from utils.constants import db_path
from config import config

psql_db = PostgresqlDatabase(
    config["database"]["postgres"]["db_name"],
    user=config["database"]["postgres"]["username"],
    password=config["database"]["postgres"]["password"],
    host=config["database"]["postgres"]["host"],
    autorollback=True,
)


def get_connections():
    conn = sqlite3.connect(f"{db_path}.db")
    cur = conn.cursor()
    return cur, conn


def close_db_connection(conn):
    conn.close()
