from db.models.users import Users


def get_user_wallets():
    return Users.select(Users.wallet_address)
