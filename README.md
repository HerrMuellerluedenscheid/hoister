Hoister 🏗
==========

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

Finally, push a new image to your registry using the same tag, and Hoister will automatically update the container.

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
