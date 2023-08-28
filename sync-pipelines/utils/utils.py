import datetime
import string
from pathlib import Path

import nanoid

from config import config, log


def get_last_synced_time(arg):
    Path(config["last_sync_time"][arg]).touch(exist_ok=True)
    with open(config["last_sync_time"][arg], "r+") as f:
        start_time = f.readline() or config['last_sync_time']['start_time'][arg]
        log.info(f"{arg} | start time -> {start_time}", )
        f.close()
        return start_time


def update_last_synced_time(arg, last_seen_time):
    with open(config["last_sync_time"][arg], "w") as f:
        f.write(str(last_seen_time))
        f.close()
    log.info(f"{arg} | update completed! -> {last_seen_time}")


def get_transaction_id():
    return config['transaction_id_prefix'] + nanoid.generate(alphabet=(string.ascii_letters + string.digits), size=6)
