import type { Cookies } from '@sveltejs/kit';
import { clerkClient } from 'svelte-clerk/server';
import {
	addProjectMember,
	claimInvitations,
	createProjectInvitation,
	deleteProjectInvitation
} from '$lib/api/projects';

/*
 * Project sharing glue between Clerk (who people are) and the controller (who
 * may see which project). The controller never talks to Clerk: this module
 * resolves emails to user ids, sends sign-up invitations, and hands the
 * controller only verified addresses when claiming invitations.
 */

/** Where Clerk sends someone who accepts a sign-up invitation. The page
 * mounts Clerk's <SignUp>, which consumes the invitation ticket. */
const SIGN_UP_PATH = '/sign-up';

/** Marks that pending invitations were already claimed for this user
 * recently, so the layout doesn't hit Clerk on every navigation. */
const CLAIM_COOKIE = 'hoister_invites_claimed';
const CLAIM_INTERVAL_SECONDS = 60 * 60;

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

type User = Awaited<ReturnType<typeof clerkClient.users.getUser>>;

export interface Person {
	id: string;
	name: string | null;
	email: string | null;
	imageUrl: string | null;
}

function verifiedEmails(user: User): string[] {
	return user.emailAddresses
		.filter((e) => e.verification?.status === 'verified')
		.map((e) => e.emailAddress.toLowerCase());
}

function person(user: User): Person {
	return {
		id: user.id,
		name: user.fullName || user.username || null,
		email: user.primaryEmailAddress?.emailAddress ?? user.emailAddresses[0]?.emailAddress ?? null,
		imageUrl: user.imageUrl || null
	};
}

/** Display info for Clerk user ids. Unknown (e.g. deleted) ids are absent. */
export async function lookupPeople(userIds: string[]): Promise<Record<string, Person>> {
	const ids = [...new Set(userIds)];
	if (ids.length === 0) return {};
	try {
		const { data } = await clerkClient.users.getUserList({ userId: ids, limit: ids.length });
		return Object.fromEntries(data.map((u) => [u.id, person(u)]));
	} catch (e) {
		console.error('[sharing] clerk user lookup failed:', e);
		return {};
	}
}

export type InviteOutcome =
	| { kind: 'added'; email: string; name: string | null }
	| { kind: 'already-member'; email: string }
	| { kind: 'invited'; email: string }
	| { kind: 'already-invited'; email: string }
	| { kind: 'error'; message: string };

/**
 * Share a project with `rawEmail`. An existing account with that verified
 * address gets access right away (and a "shared with you" email from the
 * controller); anyone else gets a pending invitation plus a Clerk sign-up
 * invitation, and gains access once they sign up with that address.
 */
export async function inviteToProject(opts: {
	inviterId: string;
	projectId: string;
	rawEmail: string;
	origin: string;
}): Promise<InviteOutcome> {
	const { inviterId, projectId, origin } = opts;
	const email = opts.rawEmail.trim().toLowerCase();
	if (!EMAIL_RE.test(email)) return { kind: 'error', message: 'Enter a valid email address.' };

	let existing: User | undefined;
	let inviterName: string | undefined;
	try {
		const [{ data: matches }, inviter] = await Promise.all([
			clerkClient.users.getUserList({ emailAddress: [email], limit: 10 }),
			clerkClient.users.getUser(inviterId)
		]);
		// The list filter is not guaranteed to be an exact match, and an
		// unverified address must not grant access to whoever typed it in.
		existing = matches.find((u) => verifiedEmails(u).includes(email));
		const me = person(inviter);
		inviterName =
			me.name && me.email ? `${me.name} (${me.email})` : (me.name ?? me.email ?? undefined);
	} catch (e) {
		console.error('[sharing] clerk lookup failed:', e);
		return { kind: 'error', message: 'Could not look up that address. Please try again.' };
	}

	if (existing) {
		if (existing.id === inviterId) {
			return { kind: 'error', message: 'That is your own address — you already own this project.' };
		}
		const result = await addProjectMember(inviterId, projectId, {
			user_id: existing.id,
			email,
			inviter: inviterName
		});
		if (!result.ok) return { kind: 'error', message: result.error };
		return result.data.added
			? { kind: 'added', email, name: person(existing).name }
			: { kind: 'already-member', email };
	}

	const invite = await createProjectInvitation(inviterId, projectId, email);
	if (!invite.ok) return { kind: 'error', message: invite.error };
	if (!invite.data.created) return { kind: 'already-invited', email };

	try {
		await clerkClient.invitations.createInvitation({
			emailAddress: email,
			redirectUrl: `${origin}${SIGN_UP_PATH}`,
			// A previous (e.g. revoked or expired) Clerk invitation for the
			// address must not block this one.
			ignoreExisting: true,
			notify: true
		});
	} catch (e) {
		console.error('[sharing] clerk invitation failed:', e);
		// Don't leave a pending invitation behind that no email points to;
		// the owner can simply try again.
		await deleteProjectInvitation(inviterId, projectId, invite.data.invitation.id).catch((err) =>
			console.error('[sharing] rollback of invitation failed:', err)
		);
		return { kind: 'error', message: 'Could not send the invitation email. Please try again.' };
	}
	return { kind: 'invited', email };
}

/**
 * Turn pending project invitations addressed to one of the user's verified
 * email addresses into memberships. Runs from the app layout; throttled with a
 * cookie so Clerk is asked at most hourly per browser, which also picks up an
 * address that was verified after the invitation arrived.
 */
export async function claimPendingInvitations(
	userId: string,
	cookies: Cookies,
	secure: boolean
): Promise<void> {
	if (cookies.get(CLAIM_COOKIE) === userId) return;
	try {
		const user = await clerkClient.users.getUser(userId);
		const emails = verifiedEmails(user);
		if (emails.length > 0) await claimInvitations(userId, emails);
		cookies.set(CLAIM_COOKIE, userId, {
			path: '/',
			httpOnly: true,
			sameSite: 'lax',
			secure,
			maxAge: CLAIM_INTERVAL_SECONDS
		});
	} catch (e) {
		// No cookie on failure, so the next page load retries.
		console.error('[sharing] claiming invitations failed:', e);
	}
}
