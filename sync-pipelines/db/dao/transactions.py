from db.models.transaction import Transactions


def get_transactions(start_time, user_wallets):
    return Transactions.select().where(Transactions.block_timestamp > start_time,
                                       Transactions.to_address << user_wallets).order_by(
        Transactions.block_timestamp.desc())
