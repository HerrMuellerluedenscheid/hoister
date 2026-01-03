import {error} from "@sveltejs/kit";
import type {Deployment} from "../../bindings/Deployment";
import {env} from '$env/dynamic/private';


const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

interface DeploymentsResponse {
    success: boolean;
    data: Deployment[];
    error: string | null;
}


export async function getDeploymentsByImage(project: string, imageName: string) {
    if (!BACKEND_URL) {
        throw error(500, 'Backend URL not configured');
    }
    const imageNameBase64 = Buffer.from(imageName).toString('base64');
    const response = await fetch(`${BACKEND_URL}/deployments/${project}/${imageNameBase64}`);

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
}
