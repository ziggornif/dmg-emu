BROWSER := firefox
COVERAGE_PATH := ./coverage/tarpaulin-report.html

.PHONY: test, coverage

test:
	cargo test

coverage:
	rm -rf ./coverage && cargo tarpaulin --out Lcov --out Html --output-dir ./coverage && $(BROWSER) $(COVERAGE_PATH)