from db.models.token_transfer import TokenTransfers


def get_token_transfers(start_time, user_addresses, support_tokens):
    return (
        TokenTransfers.select()
        .where(
            TokenTransfers.block_timestamp > start_time,
            TokenTransfers.to_address.in_(user_addresses),
            TokenTransfers.token_address.in_(support_tokens),
        )
        .order_by(TokenTransfers.block_timestamp.desc())
    )
