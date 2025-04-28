Hoister 🏗
==========

Deploy Docker images automatically with rollback support.

Add the label `hoister.enable=true` to your Docker Compose service.
Hoister checks if a new version of the image (under the same tag) is available.
It will download and start the updated container with the same settings, volumes, and networks as before.
In case of failure, it will automatically roll back to the last working state.

Installation
------------

Add the hoister.enable=true label to any service you want to manage:

```yaml
services:
  example:
    image: emrius11/example:latest
    labels:
      - "hoister.enable=true"         # <- Add this label to your service
```

Then, **either** download the [latest release](https://github.com/HerrMuellerluedenscheid/hoister/releases) that matches your OS or
add the Hoister container alongside your services:

```yaml
services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    depends_on:
      - example
```

Finally, push a new image to your registry using the same tag, and Hoister will automatically update the container.
