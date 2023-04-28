integration:
	cargo test --test features_runner -- -vv

harness:
	./test-harness.sh up

harness-down:
	./test-harness.sh down

docker-rustsdk-build:
	docker build -t rust-sdk-testing .

docker-rustsdk-run:
	docker ps -a
	docker run -it --network host rust-sdk-testing:latest

docker-test: harness docker-rustsdk-build docker-rustsdk-run
	