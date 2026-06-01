import Stripe from 'stripe';
import { env } from '$env/dynamic/private';

/**
 * Lazily build a Stripe client from `STRIPE_SECRET_KEY`. Returns `null` when
 * the key is unset so the plan page degrades gracefully (no Upgrade button)
 * instead of crashing in environments where billing isn't configured yet.
 *
 * The API version is intentionally left unset so we track the version pinned
 * to the installed `stripe` package.
 */
export function getStripe(): Stripe | null {
	const key = env.STRIPE_SECRET_KEY;
	if (!key) return null;
	return new Stripe(key);
}

/** Number of free-trial days for new subscriptions (0 = no trial). */
export function stripeTrialDays(): number {
	const n = Number(env.STRIPE_TRIAL_DAYS);
	return Number.isFinite(n) && n > 0 ? Math.floor(n) : 0;
}

/** Shape we stash on the Clerk user's private metadata. */
export interface BillingMeta {
	stripeCustomerId?: string;
	subscriptionStatus?: string;
}
