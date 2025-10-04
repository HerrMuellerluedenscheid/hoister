import type { PageServerLoad } from './$types';
import {error} from "@sveltejs/kit";
import { env } from '$env/dynamic/private';


export const load: PageServerLoad = async ({ fetch }) => {
    const backendUrl = env.HOISTER_SERVER_URL;
    const response = await fetch(`${backendUrl}/deployments`);
    if (!response.ok) {
        throw error(response.status, 'Failed to load data');
    }
    const data = await response.json();
    return { data };
};
