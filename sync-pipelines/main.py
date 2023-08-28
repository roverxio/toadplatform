import sys

from config import log
from sync.token_transfers import sync_token_transfers
from sync.transactions import sync_transactions
from utils.utils import get_last_synced_time, update_last_synced_time

if __name__ == '__main__':
    args = sys.argv[1:]
    sync = args[0]
    if sync not in ['transactions', 'token_transfers']:
        log.error('error: invalid argument')
        exit(1)

    start_time = get_last_synced_time(sync)
    end_time = start_time
    if sync == 'transactions':
        end_time = sync_transactions(start_time)
    else:
        end_time = sync_token_transfers(start_time)

    update_last_synced_time(sync, end_time)
