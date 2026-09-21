/** A newer image the agent found for one service, awaiting a manual deploy.
 * Read and applied through the project-scoped endpoints in `./projects`. */
export interface PendingUpdate {
	hostname: string;
	project_name: string;
	service_name: string;
	image_name: string;
	new_digest: string;
	detected_at: string;
}
