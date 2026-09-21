import { error } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import type { ApiResponse } from '../../bindings/ApiResponse';
import type { AddProjectMemberResponse } from '../../bindings/AddProjectMemberResponse';
import type { ClaimInvitationsResponse } from '../../bindings/ClaimInvitationsResponse';
import type { ContainerLogsResponse } from '../../bindings/ContainerLogsResponse';
import type { ContainerStateResponse } from '../../bindings/ContainerStateResponse';
import type { ContainerStateResponses } from '../../bindings/ContainerStateResponses';
import type { Deployment } from '../../bindings/Deployment';
import type { InviteToProjectResponse } from '../../bindings/InviteToProjectResponse';
import type { ProjectDetailResponse } from '../../bindings/ProjectDetailResponse';
import type { ProjectSummaryResponse } from '../../bindings/ProjectSummaryResponse';
import type { ServiceMetricsResponse } from '../../bindings/ServiceMetricsResponse';
import { backendHeaders } from './_headers';
import type { PendingUpdate } from './pendingUpdates';

/*
 * Project-scoped controller endpoints. Everything is addressed by project id
 * (not host/project name) so the same calls work for projects the user owns
 * and projects shared with them — the controller resolves access and reads
 * the owner's data.
 */

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

function base(): string {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	return BACKEND_URL;
}

function projectUrl(projectId: string, rest = ''): string {
	return `${base()}/projects/${encodeURIComponent(projectId)}${rest}`;
}

function serviceUrl(projectId: string, service: string, rest = ''): string {
	return projectUrl(projectId, `/services/${encodeURIComponent(service)}${rest}`);
}

function jsonHeaders(userId: string): Record<string, string> {
	return { ...backendHeaders(userId), 'Content-Type': 'application/json' };
}

async function unwrap<T>(response: Response, what: string): Promise<T> {
	if (!response.ok) throw error(response.status, `Failed to ${what}`);
	const result = (await response.json()) as ApiResponse<T>;
	if (!result.success || result.error || result.data == null) {
		throw error(500, result.error || `Failed to ${what}`);
	}
	return result.data;
}

/** The controller's message for a rejected write (400/403), if it sent one. */
async function rejection(response: Response, fallback: string): Promise<string> {
	const body = (await response.json().catch(() => ({}))) as { error?: string | null };
	return body.error || fallback;
}

export type WriteResult<T = null> = { ok: true; data: T } | { ok: false; error: string };

/** Every project the user owns or co-maintains, as overview tiles. */
export async function listProjects(userId: string): Promise<ProjectSummaryResponse[]> {
	const response = await fetch(`${base()}/projects`, { headers: backendHeaders(userId) });
	return unwrap<ProjectSummaryResponse[]>(response, 'load projects');
}

/** One project with its members and pending invitations; `null` when the
 * project doesn't exist or isn't shared with the user. */
export async function getProject(
	userId: string,
	projectId: string
): Promise<ProjectDetailResponse | null> {
	const response = await fetch(projectUrl(projectId), { headers: backendHeaders(userId) });
	if (response.status === 404) return null;
	return unwrap<ProjectDetailResponse>(response, 'load project');
}

/** Owner only. Returns `false` when the project is already gone. */
export async function deleteProjectById(userId: string, projectId: string): Promise<boolean> {
	const response = await fetch(projectUrl(projectId), {
		method: 'DELETE',
		headers: backendHeaders(userId)
	});
	if (response.status === 404) return false;
	if (!response.ok) throw error(response.status, 'Failed to delete project');
	return true;
}

export async function getProjectServices(
	userId: string,
	projectId: string
): Promise<ContainerStateResponses> {
	const response = await fetch(projectUrl(projectId, '/services'), {
		headers: backendHeaders(userId)
	});
	if (!response.ok) throw error(response.status, 'Failed to load services');
	return (await response.json()) as ContainerStateResponses;
}

export async function getProjectService(
	userId: string,
	projectId: string,
	service: string
): Promise<ContainerStateResponse | null> {
	const response = await fetch(serviceUrl(projectId, service), {
		headers: backendHeaders(userId)
	});
	if (response.status === 404) return null;
	return unwrap<ContainerStateResponse>(response, 'load service');
}

/** Resource-usage time series for one service over the retention window. */
export async function getProjectServiceMetrics(
	userId: string,
	projectId: string,
	service: string
): Promise<ServiceMetricsResponse> {
	const response = await fetch(serviceUrl(projectId, service, '/metrics'), {
		headers: backendHeaders(userId)
	});
	return unwrap<ServiceMetricsResponse>(response, 'load metrics');
}

/** Recent deployments of the whole project, or of one service. */
export async function getProjectDeployments(
	userId: string,
	projectId: string,
	service?: string
): Promise<Deployment[]> {
	const url = service
		? serviceUrl(projectId, service, '/deployments')
		: projectUrl(projectId, '/deployments');
	const response = await fetch(url, { headers: backendHeaders(userId) });
	return unwrap<Deployment[]>(response, 'load deployments');
}

export async function getProjectPendingUpdates(
	userId: string,
	projectId: string
): Promise<PendingUpdate[]> {
	const response = await fetch(projectUrl(projectId, '/pending-updates'), {
		headers: backendHeaders(userId)
	});
	if (!response.ok) return [];
	return (await response.json()) as PendingUpdate[];
}

/** Roll out the pending update of one service. */
export async function applyProjectUpdate(
	userId: string,
	projectId: string,
	service: string
): Promise<boolean> {
	const response = await fetch(serviceUrl(projectId, service, '/apply'), {
		method: 'POST',
		headers: backendHeaders(userId)
	});
	return response.ok;
}

/** Ask the project's agent to ship the current log tail of one service. */
export async function requestProjectServiceLogs(
	userId: string,
	projectId: string,
	service: string
): Promise<boolean> {
	const response = await fetch(serviceUrl(projectId, service, '/logs/request'), {
		method: 'POST',
		headers: backendHeaders(userId)
	});
	return response.ok;
}

/** The most recently forwarded log tail; `null` until the agent answered. */
export async function getProjectServiceLogs(
	userId: string,
	projectId: string,
	service: string
): Promise<ContainerLogsResponse | null> {
	const response = await fetch(serviceUrl(projectId, service, '/logs'), {
		headers: backendHeaders(userId)
	});
	if (!response.ok) return null;
	const body = (await response.json()) as ApiResponse<ContainerLogsResponse>;
	return body.data ?? null;
}

/**
 * Owner only: give an existing user access. `email` and `inviter` are used for
 * the "shared with you" email the controller sends when email is configured.
 */
export async function addProjectMember(
	userId: string,
	projectId: string,
	member: { user_id: string; email?: string; inviter?: string }
): Promise<WriteResult<AddProjectMemberResponse>> {
	const response = await fetch(projectUrl(projectId, '/members'), {
		method: 'POST',
		headers: jsonHeaders(userId),
		body: JSON.stringify(member)
	});
	if (response.status === 400 || response.status === 403) {
		return { ok: false, error: await rejection(response, 'Could not share the project') };
	}
	return { ok: true, data: await unwrap<AddProjectMemberResponse>(response, 'share project') };
}

/** Owner: remove anyone. Member: pass your own id to leave the project. */
export async function removeProjectMember(
	userId: string,
	projectId: string,
	memberId: string
): Promise<WriteResult> {
	const response = await fetch(projectUrl(projectId, `/members/${encodeURIComponent(memberId)}`), {
		method: 'DELETE',
		headers: backendHeaders(userId)
	});
	if (response.status === 403) {
		return { ok: false, error: await rejection(response, 'Not allowed') };
	}
	if (response.status === 404) return { ok: false, error: 'Member not found' };
	if (!response.ok) throw error(response.status, 'Failed to remove member');
	return { ok: true, data: null };
}

/** Owner only: record a pending invitation for someone without an account. */
export async function createProjectInvitation(
	userId: string,
	projectId: string,
	email: string
): Promise<WriteResult<InviteToProjectResponse>> {
	const response = await fetch(projectUrl(projectId, '/invitations'), {
		method: 'POST',
		headers: jsonHeaders(userId),
		body: JSON.stringify({ email })
	});
	if (response.status === 400 || response.status === 403) {
		return { ok: false, error: await rejection(response, 'Could not invite') };
	}
	return { ok: true, data: await unwrap<InviteToProjectResponse>(response, 'invite') };
}

export async function deleteProjectInvitation(
	userId: string,
	projectId: string,
	invitationId: string
): Promise<WriteResult> {
	const response = await fetch(
		projectUrl(projectId, `/invitations/${encodeURIComponent(invitationId)}`),
		{ method: 'DELETE', headers: backendHeaders(userId) }
	);
	if (response.status === 403) {
		return { ok: false, error: await rejection(response, 'Not allowed') };
	}
	if (response.status === 404) return { ok: false, error: 'Invitation not found' };
	if (!response.ok) throw error(response.status, 'Failed to revoke invitation');
	return { ok: true, data: null };
}

/**
 * Turn pending invitations addressed to `emails` into memberships. Only pass
 * addresses the identity provider has verified for `userId`.
 */
export async function claimInvitations(userId: string, emails: string[]): Promise<string[]> {
	const response = await fetch(`${base()}/invitations/claim`, {
		method: 'POST',
		headers: jsonHeaders(userId),
		body: JSON.stringify({ emails })
	});
	const result = await unwrap<ClaimInvitationsResponse>(response, 'claim invitations');
	return result.project_ids;
}
