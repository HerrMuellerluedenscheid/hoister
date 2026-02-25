Hoister is a tool that periodically checks for newer images and if a new image is available downloads that and updates the currently running container. In case of failure it rolls back the container update.

directories:

* agent: the client side application running as a container in a users docker compose stack. Can be used standalone without the controller
* controller: backend that allows to aggregate infos on rolled out updates and their result as well as container stats. Maybe more in the future
* frontend: connecting to the controller providing aggregated information for the user


This project uses:

 * rust
 * svelte 5
 * sqlx
 * sqlite
