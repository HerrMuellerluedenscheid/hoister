test-works:
	docker build --no-cache -f works.Dockerfile --push -t emrius11/example:latest .
	docker image rm emrius11/example:latest

test-fails:
	docker build --no-cache -f fails.Dockerfile --push -t emrius11/example:latest .
	docker image rm emrius11/example:latest
