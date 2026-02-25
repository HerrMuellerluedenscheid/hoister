import type { RequestHandler } from './$types';
import { env } from '$env/dynamic/private';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

export const POST: RequestHandler = async ({ params }) => {
  if (!BACKEND_URL) {
    return new Response('Backend URL not configured', { status: 500 });
  }

  const { hostname, project_name, service_name } = params;
  const url = `${BACKEND_URL}/pending-updates/${encodeURIComponent(hostname)}/${encodeURIComponent(project_name)}/${encodeURIComponent(service_name)}/apply`;

  const response = await fetch(url, { method: 'POST' });
  return new Response(null, { status: response.status });
};
