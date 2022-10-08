CREATE TABLE IF NOT EXISTS payment (
	 customer_id  UInt32,
	 amount       UInt32,
	 account_name Nullable(FixedString(3))
) Engine=MergeTree
ORDER BY (customer_id)
