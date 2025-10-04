.PHONY: watch

test-works:
	docker build --no-cache -f test/works.Dockerfile --push -t emrius11/example:latest .
	docker image rm emrius11/example:latest

test-fails:
	docker build --no-cache -f test/fails.Dockerfile --push -t emrius11/example:latest .
	docker image rm emrius11/example:latest

dev-frontend:
	export $$(cat .env.template | xargs) && cd frontend && npm run dev

dev-controller:
	export $$(cat .env.template | xargs) &&	cargo run --bin controller

dev-hoister:
	export $$(cat .env.template | xargs) &&	cargo run --bin hoister -- --watch 5
