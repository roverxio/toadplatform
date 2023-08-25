from db.models.token_transfer import TokenTransfers


def get_token_transfers():
    return TokenTransfers.select()
