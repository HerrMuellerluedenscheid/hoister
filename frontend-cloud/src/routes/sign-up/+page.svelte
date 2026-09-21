<script lang="ts">
	import { SignUp } from 'svelte-clerk';
</script>

<!--
	Landing page for project invitations. Clerk's invitation email links here
	with a `__clerk_ticket`, which <SignUp> picks up to create the account with
	the invited (already verified) address. The app layout then addresses the
	pending project invitation to the new account, and the projects page offers
	to accept it.
-->
<svelte:head>
	<title>Sign up · Hoister</title>
	<meta name="robots" content="noindex, nofollow" />
</svelte:head>

<main class="flex min-h-screen flex-col items-center justify-center gap-6 bg-canvas px-4 py-10">
	<div class="max-w-md text-center">
		<h1 class="text-xl font-semibold text-ink">You've been invited to Hoister</h1>
		<p class="mt-1 text-sm text-ink-muted">
			Create your account, then accept the invitation to start co-maintaining the project.
		</p>
	</div>
	<SignUp routing="hash" forceRedirectUrl="/projects" signInForceRedirectUrl="/projects" />
</main>
