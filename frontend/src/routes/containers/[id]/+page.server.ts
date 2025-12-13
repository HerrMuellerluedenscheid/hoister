
import {getDeployments} from "$lib/api/deployments";
import type {PageServerLoad} from "../../../../.svelte-kit/types/src/routes/$types";
import {getInspections} from "$lib/api/inspect";

export const load: PageServerLoad = async ({ params }) => {
    const id = params.id;
    try {
        return await getInspections(id);
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
