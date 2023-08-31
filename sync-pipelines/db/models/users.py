from peewee import CharField, BooleanField

from db.models.base import ToadBaseModel


class Users(ToadBaseModel):
    email = CharField(primary_key=True)
    wallet_address = CharField(42)
    salt = CharField()
    deployed = BooleanField()
