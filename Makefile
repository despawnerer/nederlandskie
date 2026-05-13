DOCKER_COMPOSE_FILE=docker-compose.yml

run-dev:
	docker compose -f $(DOCKER_COMPOSE_FILE) build
	docker compose -f $(DOCKER_COMPOSE_FILE) up

psql:
	docker compose -f $(DOCKER_COMPOSE_FILE) exec db psql -U postgres nederlandskie

cross-build-release:
	cross build --release
