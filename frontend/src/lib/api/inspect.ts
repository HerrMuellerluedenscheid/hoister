import {error} from "@sveltejs/kit";
import {env} from '$env/dynamic/private';
import type {ContainerStateResponses} from "../../bindings/ContainerStateResponses";


const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

export type Inspection = {
    inspections: ContainerStateResponses;
    error: string | null;
}

export async function getInspections() : Promise<Inspection> {
    if (!BACKEND_URL) {
        throw error(500, 'Backend URL not configured');
    }
    const response: Response = await fetch(`${BACKEND_URL}/container/state`);
    if (!response.ok) {
        throw error(response.status, 'Failed to load container state from backend');
    }

    const result = await response.json();

    return {
        inspections: result,
        error: null
    };
}
