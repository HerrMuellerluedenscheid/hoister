deploya
=======

Deploy docker images with rollback.

Add a label `"deploya.enable=true"` to your docker-compose service. *Deploya* checks if a new version under the same tag is available.
It will download and start the container with the same settings, volumes, networks attached as before.
In case of failure it will roll back automatically to the last working state.

```yaml
services:
  example:
    image: emrius11/example:latest
    labels:
      - "deploya.enable=true"
```
