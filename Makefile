.PHONY: coverage

coverage:
	rm -rf ./coverage && cargo tarpaulin --out Lcov --out Html --output-dir ./coverage