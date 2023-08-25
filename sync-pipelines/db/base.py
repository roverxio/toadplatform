import sqlite3

from peewee import PostgresqlDatabase

from config import config

psql_db = PostgresqlDatabase(
    config["database"]["postgres"]["db_name"],
    user=config["database"]["postgres"]["username"],
    password=config["database"]["postgres"]["password"],
    host=config["database"]["postgres"]["host"],
    autorollback=True,
)


def get_connection():
    conn = sqlite3.connect(config["database"]["storage"]["db_file"])
    cur = conn.cursor()
    return cur, conn


def close_db_connection(conn):
    conn.close()
