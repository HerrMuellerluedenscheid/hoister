import { env } from '$env/dynamic/private';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

export interface PendingUpdate {
  hostname: string;
  project_name: string;
  service_name: string;
  image_name: string;
  new_digest: string;
  detected_at: string;
}

export async function getPendingUpdates(): Promise<PendingUpdate[]> {
  if (!BACKEND_URL) return [];
  const response = await fetch(`${BACKEND_URL}/pending-updates`);
  if (!response.ok) return [];
  return (await response.json()) as PendingUpdate[];
}

export async function applyUpdate(
  hostname: string,
  projectName: string,
  serviceName: string
): Promise<void> {
  if (!BACKEND_URL) return;
  await fetch(
    `${BACKEND_URL}/pending-updates/${encodeURIComponent(hostname)}/${encodeURIComponent(projectName)}/${encodeURIComponent(serviceName)}/apply`,
    { method: 'POST' }
  );
}
