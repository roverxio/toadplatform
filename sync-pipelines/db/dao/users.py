from db.base import get_connection, close_db_connection


def get_user_wallets():
    query = "select wallet_address from users"
    cur, conn = get_connection()
    result = cur.execute(query).fetchall()
    close_db_connection(conn)
    return result
