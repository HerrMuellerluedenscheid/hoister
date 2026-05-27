import { browser } from '$app/environment';
import { env } from '$env/dynamic/public';
import posthog from 'posthog-js';

const POSTHOG_HOST = 'https://eu.i.posthog.com';

let initialized = false;

export function initPostHog(): void {
	if (!browser || initialized) return;
	const key = env.PUBLIC_POSTHOG_KEY;
	if (!key) {
		console.warn('[posthog] PUBLIC_POSTHOG_KEY not set, skipping init');
		return;
	}
	posthog.init(key, {
		api_host: POSTHOG_HOST,
		// SvelteKit handles navigation client-side; capture $pageview manually
		// so we don't miss SPA route changes and don't double-count on hard loads.
		capture_pageview: false,
		capture_pageleave: true,
		autocapture: true,
		disable_session_recording: true,
		capture_performance: true,
		persistence: 'localStorage+cookie'
	});
	initialized = true;
}

export function capturePageView(url: string): void {
	if (!initialized) return;
	posthog.capture('$pageview', { $current_url: url });
}

export function identifyUser(userId: string): void {
	if (!initialized) return;
	posthog.identify(userId);
}

export function resetUser(): void {
	if (!initialized) return;
	posthog.reset();
}

export function shutdownPostHog(): void {
	if (!initialized) return;
	posthog.reset();
	// posthog-js doesn't expose a true uninstall — opt the user out instead so
	// nothing else gets captured on the page. New init() calls will be ignored
	// by the `initialized` guard until a full reload.
	posthog.opt_out_capturing();
}
