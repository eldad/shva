.PHONY: run debug test

export RUST_LOG := shva=info

SERVICE_BASE_URL := "localhost:8042"
CURL := curl -w "\nstatus=%{http_code} %{redirect_url} size=%{size_download} time=%{time_total} content-type=\"%{content_type}\"\n"

run:
	cargo run

debug:
	RUST_LOG=debug cargo run

test:
	$(CURL) $(SERVICE_BASE_URL)/
	$(CURL) $(SERVICE_BASE_URL)/error

	$(CURL) $(SERVICE_BASE_URL)/random_error

	$(CURL) $(SERVICE_BASE_URL)/query/short
	$(CURL) $(SERVICE_BASE_URL)/query/long
