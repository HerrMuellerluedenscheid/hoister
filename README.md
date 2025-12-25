Hoister 🏗
==========

[![Discord](https://img.shields.io/discord/1453411867224576105?color=7289da&label=Discord&logo=discord&logoColor=white)](https://discord.gg/D8kHFJXY7X)

Deploy Docker images automatically with rollback support.

Add the label `hoister.enable=true` to your Docker Compose service.
Hoister checks if a new version of the image (under the same tag) is available.
It will download and start the updated container with the same settings, volumes, and networks as before.
In case of failure, it will automatically roll back to the last working state.

⚙️ Setup
--------

Add the hoister.enable=true label to any service you want to manage:

```yaml
services:
  example:
    image: emrius11/example:latest
    labels:
      - "hoister.enable=true"         # <- Add this label to your service
```

If you want hoister to also manage a containers' **named volumes** add `hoister.backup-volumes=true` as a label. On each
container update, the volumes will be backed up and restored if an update fails.

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

Finally, don't forget to push a new image to your registry using the same tag, and Hoister will automatically update the container.

## Private Registries

To access private registry, add the registry credentials to the hoister container using a sub-set of these environment variables:

```
HOISTER_REGISTRY_USERNAME
HOISTER_REGISTRY_PASSWORD
HOISTER_REGISTRY_AUTH
HOISTER_REGISTRY_EMAIL
HOISTER_REGISTRY_SERVERADDRESS
HOISTER_REGISTRY_IDENTITYTOKEN
HOISTER_REGISTRY_REGISTRYTOKEN
```

For example, to access a private image on dockerhub using a [PAT](https://docs.docker.com/security/access-tokens/)

```
HOISTER_REGISTRY_USERNAME=your-username
HOISTER_REGISTRY_PASSWORD=your-generated-personal-access-token
```

## Frontend (optional)

While the Hoister can be used as a standalone container, you can also deploy the optional frontend to manage and monitor your container updates.
Add the following service to your docker-compose.yaml:

```yaml
  hoister-controller:
    image: emrius11/hoister-controller:latest

  hoister-frontend:
    image: emrius11/hoister-frontend:latest
    ports:
      - "3000:3000"
    environment:
      HOISTER_CONTROLLER_URL: "http://hoister-controller:3033"
      HOISTER_AUTH_USERNAME: admin
      HOISTER_AUTH_PASSWORD: !a-super-secure-password!
```

Also make sure to set the `HOISTER_CONTROLLER_URL` environment variable in the Hoister container to point to the controller service.

📬 Notifications and Configuration
----------------------------------

Define the following environment variables to receive update and rollback notifications via telegram, slack or email:

```shell
export HOISTER_SLACK_WEBHOOK_URL="https://hooks.slack.com/services/XXXXXXXXX/XXXXXXXXXXXXXXXXXXXXXX"
export HOISTER_SLACK_CHANNEL="#my-update-channel"
export HOISTER_TELEGRAM_BOT_TOKEN="12345656789:XXXXXXXXXX-XXXXXXXXX-XXXXXXXXX"
export HOISTER_TELEGRAM_CHAT_ID="9999999999"
export HOISTER_CONTROLLER_URL="http://hoister-controller:3033"   # if you want to use the front end
export WATCH_INTERVAL=60   # in seconds
```

Check the [docker-compose.yaml](./docker-compose.yaml) example.
