.PHONY: watch

lint:
	cd frontend && npx prettier . --write

bindings:
	rm -rf frontend/src/bindings \
		&& cargo test export_bindings \
		&& mv -f controller/bindings frontend/src \
		&& mv shared/bindings/* frontend/src/bindings \
		&& rm -rf shared/bindings \
		&& rm -rf controller/bindings

test-works:
	docker build --no-cache -f test/works.Dockerfile --push -t emrius11/example:latest .
	docker image rm emrius11/example:latest

test-fails:
	docker build --no-cache -f test/fails.Dockerfile --push -t emrius11/example:latest .
	docker image rm emrius11/example:latest

dev-frontend: bindings
	export $$(cat .env.template | xargs) && cd frontend && npm run dev

dev-controller:
	export $$(cat .env.template | xargs) &&	cargo run --bin controller

dev-hoister:
	export $$(cat .env.template | xargs) && export $$(cat .env | xargs) && \
 	export HOISTER_CONTROLLER_URL="http://localhost:3033" && \
 	export HOISTER_SCHEDULE_INTERVAL="10" && \
 	export RUST_LOG=debug,bollard=info,hyper_util=info && \
 	HOISTER_CONTROLLER_URL="http://localhost:3033" RUST_LOG=debug,bollard=info,hyper_util=info cargo run --bin hoister

test-message:
	export $$(cat .env.template | xargs) && export $$(cat .env | xargs) &&	RUST_LOG=debug,bollard=info,hyper_util=info cargo run --bin hoister -- --test-message
