import json

from config import log
from db.base import get_connection


def create(user_transactions):
    query = ("INSERT OR IGNORE INTO user_transactions (user_address, transaction_id, from_address, to_address, amount, "
             "currency, type, status, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);")
    cur, conn = get_connection()
    log.info(user_transactions)
    cur.executemany(query, [(
        user_transaction['user_address'],
        user_transaction['transaction_id'],
        user_transaction['from_address'],
        user_transaction['to_address'],
        user_transaction['amount'],
        user_transaction['currency'],
        user_transaction['type'],
        user_transaction['status'],
        json.dumps(user_transaction['metadata']),
    ) for user_transaction in user_transactions])
    conn.commit()
    log.info(f"Created {len(user_transactions)} user transactions")
    conn.close()
