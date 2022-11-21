dev:
	sqlx db create
	sqlx migrate run
	cargo watch --exec run --workdir ./