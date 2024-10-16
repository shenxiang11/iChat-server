

test-docker:
	docker-compose -f docker-compose-test.yml up -d

clean-test-docker:
	docker-compose -f docker-compose-test.yml down

all-tests:
	cargo nextest run

coverage:
	cargo llvm-cov --html --no-cfg-coverage

docker:
	docker-compose up -d

clean-docker:
	docker-compose down
