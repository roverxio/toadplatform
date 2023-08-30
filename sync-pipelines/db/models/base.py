from peewee import *

from db.base import psql_db


class BaseModel(Model):
    class Meta:
        database = psql_db
        legacy_table_names = False
