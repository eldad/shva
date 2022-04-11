.PHONY: run debug test run_optimized

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
