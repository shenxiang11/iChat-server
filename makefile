

test-docker:
	docker-compose -f docker-compose-test.yml up -d

clean-test-docker:
	docker-compose -f docker-compose-test.yml down

all-tests:
	make clean-test-docker && make test-docker && sleep 3 && cargo nextest run && make clean-test-docker

coverage:
	make clean-test-docker && make test-docker && sleep 3 && cargo llvm-cov --html --no-cfg-coverage && make clean-test-docker

docker:
	docker-compose up -d

clean-docker:
	docker-compose down
