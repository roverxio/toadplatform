from peewee import PostgresqlDatabase

from config import config


def init_connection(db):
    return PostgresqlDatabase(
        db["db_name"],
        user=db["username"],
        password=db["password"],
        host=db["host"],
        autorollback=True,
    )


roverx_db = init_connection(config["database"]["roverx"])
toad_db = init_connection(config["database"]["toad"])
