from config import config
from db.dao.erc20_transfers import get_token_transfers
from db.dao.user_transaction import create
from db.dao.users import get_user_wallets
from utils.utils import get_transaction_id


def sync_token_transfers(start_time):
    user_transactions = []
    supported_tokens = config["erc20_contracts"]
    token_transfers = get_token_transfers(
        start_time,
        [user[0] for user in get_user_wallets()],
        list(supported_tokens.keys()),
    )
    for token_transfer in token_transfers:
        user_transactions.append(
            {
                "user_address": token_transfer.to_address,
                "transaction_id": get_transaction_id(),
                "from_address": token_transfer.from_address,
                "to_address": token_transfer.to_address,
                "amount": str(token_transfer.value),
                "currency": supported_tokens[token_transfer.token_address],
                "type": "credit",
                "status": "success",
                "metadata": {
                    "transaction_hash": token_transfer.transaction_hash,
                    "chain": config["chain"],
                    "gas_erc20": {"value": 0, "currency": ""},
                    "gas": {"value": 0, "currency": ""},
                    "from_name": "",
                    "to_name": "",
                },
            }
        )

    create(user_transactions)
    return token_transfers[0].block_timestamp if token_transfers else start_time
