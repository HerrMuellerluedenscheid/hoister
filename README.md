deploya
=======

Deploy Docker images automatically with rollback support.

Add the label "deploya.enable=true" to your Docker Compose service.
Deploya checks if a new version of the image (under the same tag) is available.
It will download and start the updated container with the same settings, volumes, and networks as before.
In case of failure, it will automatically roll back to the last working state.

Installation
------------

Add the deploya.enable=true label to any service you want to manage:

```yaml
services:
  example:
    image: emrius11/example:latest
    labels:
      - "deploya.enable=true"
```

Then, **either** download the [latest release](https://github.com/HerrMuellerluedenscheid/deploya/releases) that matches your OS or
add the Deploya container alongside your services:

```yaml
services:
  deploya:
    build: .
    image: emrius11/deploya:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    depends_on:
      - example

  example:
    build:
      context: .
      dockerfile: works.Dockerfile
    labels:
      - "deploya.enable=true"
    image: emrius11/example:latest
```

Finally, push a new image to your registry using the same tag, and Deploya will automatically update the container.