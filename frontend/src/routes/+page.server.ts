import type { PageServerLoad } from './$types';
import { error } from "@sveltejs/kit";
import { env } from '$env/dynamic/private';
import type { Deployment } from "../bindings/Deployment";
import { produce } from 'sveltekit-sse'

interface DeploymentsResponse {
    success: boolean;
    data: Deployment[];
    error: string | null;
}

export const load: PageServerLoad = async ({ fetch }) => {
    const backendUrl = env.HOISTER_CONTROLLER_URL;

    if (!backendUrl) {
        throw error(500, 'Backend URL not configured');
    }

    try {
        const response = await fetch(`${backendUrl}/deployments`);

        if (!response.ok) {
            throw error(response.status, 'Failed to load data from backend');
        }

        const result = await response.json() as DeploymentsResponse;

        if (!result.success || result.error) {
            throw error(500, result.error || 'Unknown error from backend');
        }

        return {
            deployments: result.data,
            error: null
        };
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
