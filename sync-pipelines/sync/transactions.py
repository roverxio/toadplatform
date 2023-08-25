from config import log
from db.dao.transactions import get_transactions


def sync_transactions():
    for transaction in get_transactions():
        log.info(transaction)
    log.info("hello transactions!")
