MODULE      = scraper
LIBS        = $(MODULE) tests
PYTHON      = poetry run python
PRECOMMIT   = poetry run pre-commit
# diagrams folder must be there; ex. data/NDG/diagrams
DATA_DIR    = data/S/SME
PARALLELISM ?= 3
LOCATION    ?= S
SITE        ?= SME
PLANT       ?= SME

.PHONY: clean fmt lint test init shell run-stack down-stack scrape

# first arg is mode
define run_scraper
	# this is to avoid downloading diagrams from azure locally
	# buuut, if you want to do that, you should delete this
	AZURE_BLOB_CONNECTION_STRING="" ${PYTHON} -m ${MODULE} --mode $(1) --out-dir ${DATA_DIR} \
		--diagrams-dir ${DATA_DIR}/diagrams --parallelism ${PARALLELISM} \
		--location ${LOCATION} --site ${SITE} --plant ${PLANT} \
		--excel --debug

endef

.git/hooks/pre-commit:
	@${PRECOMMIT} install
	@${PRECOMMIT} autoupdate
	@touch $@

pyproject.toml:
	@touch $@

poetry.lock: pyproject.toml
	@poetry env use "$(shell which python)"
	@poetry install
	@touch $@

init: poetry.lock .git/hooks/pre-commit

shell: poetry.lock
	@poetry shell

clean:
	@find . -type f -name "*.pyc" -delete
	@rm -f poetry.lock

lint: poetry.lock
	@${PYTHON} -m black ${LIBS}
	@${PYTHON} -m autoflake --in-place --recursive --remove-all-unused-imports --expand-star-imports ${LIBS}
	@${PYTHON} -m isort ${LIBS}
	@${PYTHON} -m mypy ${LIBS}
	@${PYTHON} -m bandit --configfile .bandit.yaml --recursive ${LIBS}

test: Earthfile
	@earthly --allow-privileged +test

run-stack: down-stack
	@docker-compose -f docker/stack.yaml --project-directory ./docker up

down-stack:
	@docker-compose -f docker/stack.yaml --project-directory ./docker down

run-neo4j: down-neo4j
	@docker-compose -f docker/neo4j.yaml --project-directory ./docker up

down-neo4j:
	@docker-compose -f docker/neo4j.yaml --project-directory ./docker down

run-full-stack: down-full-stack
	@docker-compose -f docker/stack.yaml -f docker/neo4j.yaml --project-directory ./docker up

down-full-stack:
	@docker-compose -f docker/stack.yaml -f docker/neo4j.yaml --project-directory ./docker down

scrape: poetry.lock
	@$(call run_scraper, scrape)

graph: poetry.lock
	@$(call run_scraper, graph)

draw: poetry.lock
	@$(call run_scraper, draw)
