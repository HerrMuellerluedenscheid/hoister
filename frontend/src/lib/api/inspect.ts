import {error} from "@sveltejs/kit";
import {env} from '$env/dynamic/private';


const BACKEND_URL = env.HOISTER_CONTROLLER_URL;


export async function getInspections(id: string | null = null) {
    if (!BACKEND_URL) {
        throw error(500, 'Backend URL not configured');
    }

    let response: Response;
    if (id !== null) {
        response = await fetch(`${BACKEND_URL}/container/state/${id}`);
    } else {
        response = await fetch(`${BACKEND_URL}/container/state`);
    }
    if (!response.ok) {
        throw error(response.status, 'Failed to load container state from backend');
    }

    const result = await response.json();
    const inspections = result.data["container_inspections"];
    return {
        inspections: inspections,
        error: null
    };
}
