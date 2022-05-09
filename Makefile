.PHONY: run debug infra test run_optimized fmt openapi migrate check-migrations verify-migration-versioning

export RUST_LOG := shva=info

SERVICE_BASE_URL := "localhost:8042"
CURL := curl -H 'x-auth-api-key:apikey1' --compressed -w "\nstatus=%{http_code} %{redirect_url} size=%{size_download} time=%{time_total} content-type=\"%{content_type}\"\n"

infra:
	docker-compose up -d

run: infra
	RUST_LOG=info cargo run

run_optimized: infra
	RUST_LOG=info cargo run --release

debug:
	RUST_LOG=debug cargo run

test:
	# 404
	$(CURL) $(SERVICE_BASE_URL)/no-such-page

	$(CURL) $(SERVICE_BASE_URL)/monitoring/metrics

	$(CURL) $(SERVICE_BASE_URL)/
	$(CURL) $(SERVICE_BASE_URL)/error

	$(CURL) $(SERVICE_BASE_URL)/random-error

	$(CURL) $(SERVICE_BASE_URL)/query/short
	$(CURL) $(SERVICE_BASE_URL)/query/long


fmt:
	# Workaround for using unstable rustfmt features.
	# When features are available in stable, move the options into rustfmt.toml and remove this.
	cargo fmt -- --config imports_granularity=Crate,group_imports=StdExternalCrate

openapi:
	cargo --quiet run -- openapi

migrate:
	cargo --quiet run -- migrate

check-migrations:
	cargo --quiet run -- check-migrations

verify-migration-versioning:
	cargo --quiet run -- verify-migration-versioning

doc:
	cargo doc -p shva --no-deps --open