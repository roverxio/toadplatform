from peewee import CharField, BooleanField

from db.models.base import BaseModel


class Users(BaseModel):
    email = CharField(primary_key=True)
    wallet_address = CharField(42)
    salt = CharField()
    deployed = BooleanField()
