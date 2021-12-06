integration:
	cargo test --test cucumber --

docker-test:
	./tests/docker/run_docker.sh
