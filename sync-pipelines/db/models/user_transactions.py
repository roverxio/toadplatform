from peewee import CharField, IntegerField
from playhouse.postgres_ext import DateTimeTZField, JSONField

from db.models.base import ToadBaseModel


class UserTransactions(ToadBaseModel):
    id = IntegerField(primary_key=True)
    user_address = CharField(42)
    transaction_id = CharField()
    from_address = CharField(42)
    to_address = CharField(42)
    amount = CharField()
    currency = CharField()
    type = CharField(6)
    status = CharField(10)
    metadata = JSONField()
    created_at = DateTimeTZField()
    updated_at = DateTimeTZField()
