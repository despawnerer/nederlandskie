DOCKER_COMPOSE_FILE=docker-compose.yml

run-dev:
	docker compose -f $(DOCKER_COMPOSE_FILE) build
	docker compose -f $(DOCKER_COMPOSE_FILE) up

cross-build-release:
	cross build --release
