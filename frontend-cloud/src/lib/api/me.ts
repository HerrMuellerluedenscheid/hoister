import { error } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import { backendHeaders } from './_headers';
import type { NotifierKind } from './notifiers';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

export type Plan = 'free' | 'pro';

export interface PlanLimits {
	/** null = unlimited */
	max_projects: number | null;
	allowed_notifier_kinds: NotifierKind[];
}

export interface Usage {
	projects: number;
	notifiers_by_kind: Record<string, number>;
}

export interface PlanStatus {
	plan: Plan;
	limits: PlanLimits;
	usage: Usage;
}

interface ApiResponse<T> {
	success: boolean;
	data: T | null;
	error: string | null;
}

export async function getMe(userId: string): Promise<PlanStatus> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const response = await fetch(`${BACKEND_URL}/me`, {
		headers: backendHeaders(userId)
	});
	if (!response.ok) throw error(response.status, 'Failed to load plan');
	const result = (await response.json()) as ApiResponse<PlanStatus>;
	if (!result.success || result.error || !result.data) {
		throw error(500, result.error || 'Failed to load plan');
	}
	return result.data;
}
