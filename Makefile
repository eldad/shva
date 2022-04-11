.PHONY: run debug test run_optimized fmt openapi migrate

export RUST_LOG := shva=info

SERVICE_BASE_URL := "localhost:8042"
CURL := curl -H 'x-auth-api-key:apikey1' --compressed -w "\nstatus=%{http_code} %{redirect_url} size=%{size_download} time=%{time_total} content-type=\"%{content_type}\"\n"

run:
	docker-compose up -d
	cargo run

debug:
	RUST_LOG=debug cargo run

test:
	$(CURL) $(SERVICE_BASE_URL)/metrics

	$(CURL) $(SERVICE_BASE_URL)/
	$(CURL) $(SERVICE_BASE_URL)/error

	$(CURL) $(SERVICE_BASE_URL)/random_error

	$(CURL) $(SERVICE_BASE_URL)/query/short
	$(CURL) $(SERVICE_BASE_URL)/query/long

run_optimized:
	cargo run --release

fmt:
	# Workaround for using unstable rustfmt features.
	# When features are available in stable, move the options into rustfmt.toml and remove this.
	cargo fmt -- --config imports_granularity=Crate,group_imports=StdExternalCrate

openapi:
	cargo --quiet run -- openapi

migrate:
	cargo --quiet run -- migrate