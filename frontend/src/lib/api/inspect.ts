import {error} from "@sveltejs/kit";
import {env} from '$env/dynamic/private';


const BACKEND_URL = env.HOISTER_CONTROLLER_URL;


export async function getInspections() {
    if (!BACKEND_URL) {
        throw error(500, 'Backend URL not configured');
    }

    const response = await fetch(`${BACKEND_URL}/container/state`);

    if (!response.ok) {
        throw error(response.status, 'Failed to load container state from backend');
    }
    const result = await response.json() as [any];
    console.info(JSON.stringify(result[0], null, 2));
    return {
        inspections: result,
        error: null
    };
}
