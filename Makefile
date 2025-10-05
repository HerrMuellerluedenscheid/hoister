.PHONY: watch

generate-bindings:
	rm -rf frontend/src/bindings && cargo test export_bindings && mv -f controller/bindings frontend/src

test-works:
	docker build --no-cache -f test/works.Dockerfile --push -t emrius11/example:latest .
	docker image rm emrius11/example:latest

test-fails:
	docker build --no-cache -f test/fails.Dockerfile --push -t emrius11/example:latest .
	docker image rm emrius11/example:latest

dev-frontend: generate-bindings
	export $$(cat .env.template | xargs) && cd frontend && npm run dev

dev-controller:
	export $$(cat .env.template | xargs) &&	cargo run --bin controller

dev-hoister:
	export $$(cat .env.template | xargs) && export $$(cat .env | xargs) &&	RUST_LOG=debug,bollard=info cargo run --bin hoister -- --watch 5
