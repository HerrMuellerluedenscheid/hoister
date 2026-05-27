import { browser } from '$app/environment';

export type ConsentState = 'unknown' | 'accepted' | 'declined';

const STORAGE_KEY = 'analytics-consent';

function read(): ConsentState {
	if (!browser) return 'unknown';
	const raw = localStorage.getItem(STORAGE_KEY);
	return raw === 'accepted' || raw === 'declined' ? raw : 'unknown';
}

let consent = $state<ConsentState>(read());

export const analyticsConsent = {
	get value(): ConsentState {
		return consent;
	},
	accept() {
		consent = 'accepted';
		if (browser) localStorage.setItem(STORAGE_KEY, 'accepted');
	},
	decline() {
		consent = 'declined';
		if (browser) localStorage.setItem(STORAGE_KEY, 'declined');
	}
};
