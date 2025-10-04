import type { PageServerLoad } from './$types';
import {error} from "@sveltejs/kit";
import { env } from '$env/dynamic/private';
import type {Deployment} from "../bindings/Deployment";

export const load: PageServerLoad = async ({ fetch }) => {
    const backendUrl = env.HOISTER_SERVER_URL;
    const response = await fetch(`${backendUrl}/deployments`);
    if (!response.ok) {
        throw error(response.status, 'Failed to load data');
    }
    const data = await response.json() as { success: boolean; data: Deployment[] ; error: string | null};
    return { data };
};
