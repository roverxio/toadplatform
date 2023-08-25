from db.models.transaction import Transactions


def get_transactions():
    return Transactions.select()
