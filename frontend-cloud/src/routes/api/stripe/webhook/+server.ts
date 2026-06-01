import { error, json } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import { clerkClient } from 'svelte-clerk/server';
import { getStripe } from '$lib/server/stripe';
import { setPlan } from '$lib/api/billing';
import type { RequestHandler } from './$types';
import type Stripe from 'stripe';

/**
 * Stripe webhook receiver. This is the source of truth for plan changes:
 * Checkout success and subscription lifecycle events flip the controller's
 * stored plan between `pro` and `free`. The signature is verified against
 * `STRIPE_WEBHOOK_SECRET` before anything is trusted.
 *
 * Public route (outside the `(app)` group) — Stripe is an unauthenticated
 * external caller, authenticated instead by the webhook signature.
 */
export const POST: RequestHandler = async ({ request }) => {
	const stripe = getStripe();
	const secret = env.STRIPE_WEBHOOK_SECRET;
	if (!stripe || !secret) throw error(503, 'Stripe webhook is not configured');

	const sig = request.headers.get('stripe-signature');
	if (!sig) throw error(400, 'Missing stripe-signature header');

	// Raw body is required for signature verification.
	const payload = await request.text();

	let event: Stripe.Event;
	try {
		event = await stripe.webhooks.constructEventAsync(payload, sig, secret);
	} catch (e) {
		console.error('[stripe webhook] signature verification failed:', e);
		throw error(400, 'Invalid signature');
	}

	try {
		await handleEvent(stripe, event);
	} catch (e) {
		// Returning 500 makes Stripe retry, which is what we want for transient
		// controller/Clerk failures.
		console.error(`[stripe webhook] handling ${event.type} failed:`, e);
		throw error(500, 'Webhook handler error');
	}

	return json({ received: true });
};

async function handleEvent(stripe: Stripe, event: Stripe.Event): Promise<void> {
	switch (event.type) {
		case 'checkout.session.completed': {
			const session = event.data.object as Stripe.Checkout.Session;
			const userId = session.client_reference_id ?? session.metadata?.user_id ?? null;
			const customerId = customerIdOf(session.customer);
			if (userId) await persist(userId, customerId, 'active', 'pro');
			break;
		}
		case 'customer.subscription.created':
		case 'customer.subscription.updated': {
			const sub = event.data.object as Stripe.Subscription;
			const userId = sub.metadata?.user_id ?? (await userIdFromCustomer(stripe, sub.customer));
			if (userId) {
				const plan = isActiveStatus(sub.status) ? 'pro' : 'free';
				await persist(userId, customerIdOf(sub.customer), sub.status, plan);
			}
			break;
		}
		case 'customer.subscription.deleted': {
			const sub = event.data.object as Stripe.Subscription;
			const userId = sub.metadata?.user_id ?? (await userIdFromCustomer(stripe, sub.customer));
			if (userId) await persist(userId, customerIdOf(sub.customer), 'canceled', 'free');
			break;
		}
		default:
			// Ignored: invoice.*, payment_intent.*, etc. Subscription state is the
			// authority for plan; Stripe's own retries handle dunning.
			break;
	}
}

/** trialing/active keep Pro; past_due keeps Pro during Stripe's retry grace.
 *  A cancel-at-period-end stays `active` until Stripe emits
 *  `subscription.deleted`, which is when we drop to Free. */
function isActiveStatus(status: Stripe.Subscription.Status): boolean {
	return status === 'active' || status === 'trialing' || status === 'past_due';
}

function customerIdOf(
	customer: string | Stripe.Customer | Stripe.DeletedCustomer | null
): string | undefined {
	if (!customer) return undefined;
	return typeof customer === 'string' ? customer : customer.id;
}

async function userIdFromCustomer(
	stripe: Stripe,
	customer: string | Stripe.Customer | Stripe.DeletedCustomer
): Promise<string | null> {
	const id = customerIdOf(customer);
	if (!id) return null;
	try {
		const c = await stripe.customers.retrieve(id);
		if (!c.deleted) return (c.metadata?.user_id as string | undefined) ?? null;
	} catch (e) {
		console.error('[stripe webhook] customer lookup failed:', e);
	}
	return null;
}

async function persist(
	userId: string,
	customerId: string | undefined,
	status: string,
	plan: 'pro' | 'free'
): Promise<void> {
	// Controller plan store first — it's what actually gates the product.
	await setPlan(userId, plan);
	// Then stash the Stripe customer id + status on the Clerk user so the plan
	// page can show status and open the billing portal. Best-effort.
	try {
		await clerkClient.users.updateUserMetadata(userId, {
			privateMetadata: {
				...(customerId ? { stripeCustomerId: customerId } : {}),
				subscriptionStatus: status
			}
		});
	} catch (e) {
		console.error('[stripe webhook] clerk metadata update failed:', e);
	}
}
