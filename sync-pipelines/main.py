import sys

from config import log
from sync.token_transfers import sync_token_transfers
from sync.transactions import sync_transactions

if __name__ == '__main__':
    args = sys.argv[1:]
    sync = args[0]
    if sync not in ['transactions', 'token_transfers']:
        log.error('error: invalid argument')
        exit(1)

    if sync == 'transactions':
        sync_transactions()
    else:
        sync_token_transfers()
