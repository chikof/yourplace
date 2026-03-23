DROP INDEX IF EXISTS idx_coin_transactions_user;
DROP TABLE IF EXISTS coin_transactions;
ALTER TABLE users DROP COLUMN IF EXISTS coins;
