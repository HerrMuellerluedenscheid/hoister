
import type {PageServerLoad} from "../../../../.svelte-kit/types/src/routes/$types";
import {getInspections} from "$lib/api/inspect";
import {getDeploymentsByImage} from "$lib/api/deploymentsByImage";

export const load: PageServerLoad = async ({ params }) => {
    const id = params.id;
    try {
        const inspections = await getInspections(id);

        const project = inspections.inspections[0].Config.Labels["com.docker.compose.project"];
        const imageName = inspections.inspections[0].Config.Image.split(":")[0];
        const deployments = await getDeploymentsByImage(project, imageName);
        // return {...inspections, "deployments": deployments};
        return inspections
    } catch (err) {
        console.error('Failed to load deployments:', err);

        // If it's already a SvelteKit error, re-throw it
        if (err && typeof err === 'object' && 'status' in err) {
            throw err;
        }

        // Otherwise, return empty data with error message
        return {
            deployments: [],
            error: 'Failed to connect to backend service'
        };
    }
};
