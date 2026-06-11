import { error, fail, redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import { clerkClient } from 'svelte-clerk/server';
import type { Actions, PageServerLoad } from './$types';
import { getMe } from '$lib/api/me';
import { getStripe, type BillingMeta } from '$lib/server/stripe';
import { startProCheckout } from '$lib/server/checkout';

async function billingMeta(userId: string): Promise<BillingMeta> {
	try {
		const user = await clerkClient.users.getUser(userId);
		return (user.privateMetadata ?? {}) as BillingMeta;
	} catch (e) {
		console.error('[plan] clerk getUser failed:', e);
		return {};
	}
}

export const load: PageServerLoad = async ({ locals, url }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	const stripeReady = Boolean(env.STRIPE_SECRET_KEY && env.STRIPE_PRICE_ID);
	const meta = await billingMeta(auth.userId);

	try {
		const me = await getMe(auth.userId);
		return {
			me,
			meError: null,
			stripeReady,
			subscriptionStatus: meta.subscriptionStatus ?? null,
			hasCustomer: Boolean(meta.stripeCustomerId),
			checkout: url.searchParams.get('checkout')
		};
	} catch (e) {
		console.error('[plan] load failed:', e);
		return {
			me: null,
			meError: 'Failed to load plan from the controller',
			stripeReady,
			subscriptionStatus: null,
			hasCustomer: false,
			checkout: url.searchParams.get('checkout')
		};
	}
};

export const actions: Actions = {
	// Start a Stripe Checkout session for the Pro subscription and redirect to
	// Stripe's hosted page.
	upgrade: async ({ locals, url }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const result = await startProCheckout(auth.userId, url.origin);
		if (!result.ok) return fail(result.status, { error: result.error });
		throw redirect(303, result.url);
	},

	// Open the Stripe Billing Portal so the user can manage / cancel.
	manage: async ({ locals, url }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const stripe = getStripe();
		if (!stripe) return fail(503, { error: 'Billing is not configured yet.' });

		const meta = await billingMeta(auth.userId);
		if (!meta.stripeCustomerId) return fail(400, { error: 'No subscription to manage yet.' });

		let portalUrl: string | null = null;
		try {
			const session = await stripe.billingPortal.sessions.create({
				customer: meta.stripeCustomerId,
				return_url: `${url.origin}/settings/plan`
			});
			portalUrl = session.url;
		} catch (e) {
			console.error('[plan] portal session create failed:', e);
			return fail(502, { error: 'Could not open the billing portal. Please try again.' });
		}

		throw redirect(303, portalUrl);
	}
};
