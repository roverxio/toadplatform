from config import log
from db.dao.erc20_transfers import get_token_transfers


def sync_token_transfers():
    log.info(get_token_transfers())
    for token_transfer in get_token_transfers():
        log.info(token_transfer)
    log.info("hello token_transfers!")
