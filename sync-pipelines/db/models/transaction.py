from peewee import CharField, BigIntegerField
from playhouse.postgres_ext import DateTimeTZField

from db.models.base import RoverXBaseModel


class Transactions(RoverXBaseModel):
    hash = CharField(primary_key=True)
    block_number = None
    from_address = CharField()
    to_address = CharField()
    value = BigIntegerField()
    block_timestamp = DateTimeTZField()
