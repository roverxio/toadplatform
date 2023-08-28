from peewee import *
from playhouse.postgres_ext import DateTimeTZField

from .base import BaseModel


class TokenTransfers(BaseModel):
    transaction_hash = CharField(primary_key=True)
    token_address = CharField()
    from_address = CharField()
    to_address = CharField()
    value = BigIntegerField()
    block_timestamp = DateTimeTZField()
