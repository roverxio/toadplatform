from config import config
from db.dao.transactions import get_transactions
from db.dao.user_transaction import create
from db.dao.users import get_user_wallets
from utils.utils import get_transaction_id


def sync_transactions(start_time):
    user_transactions = []
    transactions = get_transactions(
        start_time, [user[0] for user in get_user_wallets()]
    )
    for transaction in transactions:
        user_transactions.append(
            {
                "user_address": transaction.to_address,
                "transaction_id": get_transaction_id(),
                "from_address": transaction.from_address,
                "to_address": transaction.to_address,
                "amount": str(transaction.value),
                "currency": config["native_currency"],
                "type": "credit",
                "status": "success",
                "metadata": {
                    "transaction_hash": transaction.hash,
                    "chain": config["chain"],
                    "gas_erc20": {"value": 0, "currency": ""},
                    "gas": {"value": 0, "currency": ""},
                    "from_name": "",
                    "to_name": "",
                },
            }
        )

    create(user_transactions)
    return transactions[0].block_timestamp if transactions else start_time
