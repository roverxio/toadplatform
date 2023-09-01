from config import log
from db.models.user_transactions import UserTransactions


def create(user_transactions):
    UserTransactions.insert_many(user_transactions, [
        UserTransactions.user_address,
        UserTransactions.transaction_id,
        UserTransactions.from_address,
        UserTransactions.to_address,
        UserTransactions.amount,
        UserTransactions.currency,
        UserTransactions.type,
        UserTransactions.status,
        UserTransactions.metadata,
    ]).execute()
    log.info(f"Created {len(user_transactions)} user transactions")
