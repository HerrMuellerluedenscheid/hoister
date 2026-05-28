import { browser } from '$app/environment';
import { env } from '$env/dynamic/public';
import type { PostHog } from 'posthog-js';

const POSTHOG_HOST = 'https://eu.i.posthog.com';

let posthog: PostHog | null = null;
let initializing: Promise<void> | null = null;

async function ensureLoaded(): Promise<PostHog | null> {
	if (!browser) return null;
	if (posthog) return posthog;
	if (!initializing) {
		initializing = (async () => {
			const mod = await import('posthog-js');
			posthog = mod.default;
		})();
	}
	await initializing;
	return posthog;
}

export async function initPostHog(): Promise<void> {
	if (!browser) return;
	const key = env.PUBLIC_POSTHOG_KEY;
	if (!key) {
		console.warn('[posthog] PUBLIC_POSTHOG_KEY not set, skipping init');
		return;
	}
	const ph = await ensureLoaded();
	if (!ph) return;
	if (ph.__loaded) return;
	ph.init(key, {
		api_host: POSTHOG_HOST,
		// SvelteKit handles navigation client-side; capture $pageview manually
		// so we don't miss SPA route changes and don't double-count on hard loads.
		capture_pageview: false,
		capture_pageleave: true,
		// Autocapture is restricted to clicks and submits — we deliberately
		// drop `change`/`input` events because the notifier creation form
		// (and any future credential form) has plain text inputs that
		// would otherwise ship the user's bot tokens and webhook URLs to
		// PostHog on every keystroke. Forms that hold secrets carry
		// `class="ph-no-capture"` as a second line of defence.
		autocapture: {
			dom_event_allowlist: ['click', 'submit'],
			element_allowlist: ['a', 'button', 'form', 'select', 'label']
		},
		disable_session_recording: true,
		capture_performance: true,
		persistence: 'localStorage+cookie'
	});
}

export function capturePageView(url: string): void {
	if (!posthog?.__loaded) return;
	posthog.capture('$pageview', { $current_url: url });
}

export function identifyUser(userId: string): void {
	if (!posthog?.__loaded) return;
	posthog.identify(userId);
}

export function resetUser(): void {
	if (!posthog?.__loaded) return;
	posthog.reset();
}

export function shutdownPostHog(): void {
	if (!posthog?.__loaded) return;
	posthog.reset();
	// posthog-js doesn't expose a true uninstall — opt the user out instead so
	// nothing else gets captured on the page. New init() calls will be ignored
	// by the `__loaded` guard until a full reload.
	posthog.opt_out_capturing();
}
