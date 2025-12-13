import type {PageServerLoad} from './$types';

import {getDeployments} from "$lib/api/deployments";

export const load: PageServerLoad = async ({ fetch }) => {


    try {
        return await getDeployments();
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
