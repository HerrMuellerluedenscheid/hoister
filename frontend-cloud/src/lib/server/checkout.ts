import { env } from '$env/dynamic/private';
import { clerkClient } from 'svelte-clerk/server';
import { getStripe, stripeTrialDays, type BillingMeta } from './stripe';

export type CheckoutResult =
	| { ok: true; url: string }
	| { ok: false; status: number; error: string };

/**
 * Start a Stripe Checkout session for the Pro subscription and return its
 * hosted URL. Shared by every "Upgrade to Pro" entry point (the plan page and
 * the project-limit banner) so they all reuse the same customer lookup, trial
 * handling, and success/cancel redirects.
 *
 * `origin` is the request origin used to build the return URLs; checkout always
 * sends the user back to `/settings/plan` so the result surfaces in one place.
 */
export async function startProCheckout(userId: string, origin: string): Promise<CheckoutResult> {
	const stripe = getStripe();
	const priceId = env.STRIPE_PRICE_ID;
	if (!stripe || !priceId)
		return { ok: false, status: 503, error: 'Billing is not configured yet.' };

	// Reuse an existing customer if we have one; otherwise prefill the email.
	let email: string | undefined;
	let customerId: string | undefined;
	try {
		const user = await clerkClient.users.getUser(userId);
		email = user.emailAddresses.find((e) => e.id === user.primaryEmailAddressId)?.emailAddress;
		customerId = (user.privateMetadata as BillingMeta)?.stripeCustomerId;
	} catch (e) {
		console.error('[checkout] clerk lookup failed:', e);
	}

	const trialDays = stripeTrialDays();
	try {
		const session = await stripe.checkout.sessions.create({
			mode: 'subscription',
			line_items: [{ price: priceId, quantity: 1 }],
			client_reference_id: userId,
			...(customerId ? { customer: customerId } : email ? { customer_email: email } : {}),
			subscription_data: {
				metadata: { user_id: userId },
				...(trialDays > 0 ? { trial_period_days: trialDays } : {})
			},
			success_url: `${origin}/settings/plan?checkout=success`,
			cancel_url: `${origin}/settings/plan?checkout=cancelled`
		});
		if (!session.url)
			return { ok: false, status: 502, error: 'Stripe did not return a checkout URL.' };
		return { ok: true, url: session.url };
	} catch (e) {
		console.error('[checkout] checkout session create failed:', e);
		return { ok: false, status: 502, error: 'Could not start checkout. Please try again.' };
	}
}
