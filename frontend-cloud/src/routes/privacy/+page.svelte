<script lang="ts">
	import SEO from '$lib/components/SEO.svelte';
</script>

<SEO
	title="Privacy Policy — Hoister"
	description="Privacy policy for Hoister."
	path="/privacy"
	noindex
/>

<div class="flex min-h-screen flex-col bg-zinc-950 text-zinc-100">
	<header class="flex items-center justify-between px-8 py-5">
		<a href="/" class="flex items-center gap-2">
			<svg
				class="h-7 w-7 text-indigo-400"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M5 10l1.5-4.5h11L19 10M5 10h14M5 10l-2 7h18l-2-7M9 17v2m6-2v2m-3-9v4"
				/>
			</svg>
			<span class="text-lg font-semibold tracking-tight">Hoister</span>
		</a>
		<a href="/datenschutz" class="text-sm text-zinc-500 hover:text-zinc-300">Deutsch</a>
	</header>

	<main class="mx-auto w-full max-w-2xl flex-1 px-8 py-12">
		<h1 class="mb-2 text-3xl font-bold tracking-tight">Privacy Policy</h1>
		<p class="mb-4 text-sm text-zinc-500">Last updated: June 2026</p>
		<p class="mb-10 text-sm text-zinc-500">
			This is an English translation provided for convenience. The legally binding version is the
			German
			<a href="/datenschutz" class="text-indigo-400 hover:text-indigo-300">Datenschutzerklärung</a>.
		</p>

		<!-- 1 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				1. Data Controller
			</h2>
			<div class="space-y-1 text-zinc-300">
				<p>Vector &amp; Veneer UG (haftungsbeschränkt)</p>
				<p>Rubensstr. 108, 12157 Berlin, Germany</p>
				<p>
					Email: <a
						href="mailto:info@vectorandveneer.com"
						class="text-indigo-400 hover:text-indigo-300">info@vectorandveneer.com</a
					>
				</p>
			</div>
		</section>

		<!-- 2 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				2. What Data We Process
			</h2>
			<p class="mb-3 text-zinc-400">
				The following personal data is processed when operating this service:
			</p>
			<ul class="list-disc space-y-2 pl-5 text-zinc-400">
				<li>
					<span class="text-zinc-300">Server log data</span> – IP address, date and time of the request,
					requested URL, HTTP status code, amount of data transferred, referrer, browser and operating-system
					identifier. This data is collected automatically when the website is accessed.
				</li>
				<li>
					<span class="text-zinc-300">Account data</span> – email address and OAuth profile information
					(name, profile picture) that you provide when signing in via GitHub or Google.
				</li>
				<li>
					<span class="text-zinc-300">Authentication session data</span> – encrypted session tokens
					stored in cookies to keep you signed in during a session.
				</li>
				<li>
					<span class="text-zinc-300">Notification integration configuration</span> – the credentials
					you provide to connect a messaging service (e.g. Slack webhook URLs, Discord bot tokens or
					webhook URLs, channel identifiers, email recipients). See Section 6.
				</li>
			</ul>
		</section>

		<!-- 3 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				3. Purposes and Legal Bases of Processing
			</h2>
			<div class="space-y-4 text-zinc-400">
				<div>
					<p class="mb-1 font-medium text-zinc-300">Server log data</p>
					<p>
						Purpose: ensuring technically error-free operation and detecting attacks. Legal basis:
						Art. 6(1)(f) GDPR (legitimate interest). Retention: a maximum of 14 days, after which it
						is automatically deleted.
					</p>
				</div>
				<div>
					<p class="mb-1 font-medium text-zinc-300">Account data &amp; authentication</p>
					<p>
						Purpose: providing the user account and the service's features (e.g. dashboard,
						deployment management). Legal basis: Art. 6(1)(b) GDPR (performance of a contract or
						pre-contractual measures). Retention: until the account is deleted.
					</p>
				</div>
			</div>
		</section>

		<!-- 4 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				4. Authentication Service: Clerk
			</h2>
			<p class="mb-3 text-zinc-400">
				For user management and authentication we use the service <span class="text-zinc-300"
					>Clerk</span
				> (Clerk, Inc., 2 Park Avenue, Floor 20, New York, NY 10016, USA).
			</p>
			<p class="mb-3 text-zinc-400">
				Clerk processes your name, email address and authentication tokens on our instructions as a
				processor pursuant to Art. 28 GDPR. A data processing agreement (DPA) is in place.
			</p>
			<p class="text-zinc-400">
				As Clerk is based in the USA, data is transferred to a third country. The transfer is based
				on the European Commission's Standard Contractual Clauses (SCCs) pursuant to Art. 46(2)(c)
				GDPR. More information: <a
					href="https://clerk.com/legal/privacy"
					class="text-indigo-400 hover:text-indigo-300"
					target="_blank"
					rel="noopener noreferrer">clerk.com/legal/privacy</a
				>.
			</p>
		</section>

		<!-- 5 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				5. Telemetry Transmitted by the Hoister Agent
			</h2>
			<p class="mb-3 text-zinc-400">
				If you configure the Hoister agent (<code
					class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">hoister/hoister</code
				>) to connect to this hosted controller instance (<code
					class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs"
					>HOISTER_CONTROLLER_URL=https://api.hoister.io</code
				>), the agent periodically transmits the following data to your account:
			</p>
			<ul class="list-disc space-y-2 pl-5 text-zinc-400">
				<li>
					<span class="text-zinc-300">Container metadata</span> – image names (including private
					registry paths), container labels, mount paths on the host, IP addresses from
					Docker-managed networks, command-line arguments, and the configuration and status fields
					returned by the Docker inspect endpoint.
				</li>
				<li>
					<span class="text-zinc-300">Environment variable keys</span> – the agent transmits the
					<em>names</em> of your environment variables (e.g.
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">INTERNAL_API_HOST</code>).
					<span class="text-zinc-300">Values</span> of variables whose key suggests a sensitive
					meaning (<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">password</code>,
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">token</code>,
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">secret</code>,
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">key</code>,
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">webhook</code> and the
					like) are replaced with
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">***REDACTED***</code>
					in the agent before transmission.
				</li>
				<li>
					<span class="text-zinc-300">Deployment events</span> – successful or failed container
					updates, including image digest, hostname, project and service name.
				</li>
				<li>
					<span class="text-zinc-300">Container log excerpts (opt-in only)</span> –
					<strong>disabled</strong> by default. If the agent is started with
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs"
						>HOISTER_REPORT_LOGS=true</code
					>
					and a container is in the <em>restarting</em>, <em>exited</em> or <em>dead</em> state, the
					agent transmits the last ~50 lines of container log (maximum 16 KB). Values of sensitive
					environment variables are stripped from the log before transmission. Despite this
					heuristic, logs may still contain sensitive data — only enable this option if you accept
					that.
				</li>
			</ul>
			<p class="mt-4 text-zinc-400">
				<span class="text-zinc-300">Purpose:</span> displaying the status of your containers and
				deployment history in your dashboard.<br />
				<span class="text-zinc-300">Legal basis:</span> Art. 6(1)(b) GDPR (performance of a
				contract).<br />
				<span class="text-zinc-300">Storage location:</span> EU (servers in Germany).<br />
				<span class="text-zinc-300">Retention:</span> container states are held in memory and are
				lost after a restart of the controller instance. Deployment records are kept until your
				account is deleted.<br />
				<span class="text-zinc-300">Tenant isolation:</span> all transmitted data is stored
				exclusively under your user ID and is technically inaccessible to other users.
			</p>
			<p class="mt-3 text-zinc-400">
				You can object to this processing at any time by running the agent without a
				<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs"
					>HOISTER_CONTROLLER_TOKEN</code
				>
				(standalone mode, no telemetry) or by setting up your own controller instance (self-hosted
				mode). Details:
				<a
					href="https://docs.hoister.io/guides/operating-modes/"
					class="text-indigo-400 hover:text-indigo-300"
					target="_blank"
					rel="noopener noreferrer">docs.hoister.io/guides/operating-modes</a
				>.
			</p>
		</section>

		<!-- 6 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				6. Notification Integrations (Slack, Discord, Email and More)
			</h2>
			<p class="mb-3 text-zinc-400">
				Optionally, you can connect one or more messaging services to your account so that Hoister
				notifies you about deployment events (available updates, successful updates, rollbacks).
				Supported services include <span class="text-zinc-300"
					>Slack, Discord, Telegram, Microsoft Teams, Matrix, Mattermost, Rocket.Chat, Google Chat,
					Gotify, ntfy, Pushover, email, and generic webhooks</span
				>. This feature is optional and only active if you set up an integration.
			</p>

			<p class="mb-1 font-medium text-zinc-300">What we store</p>
			<p class="mb-3 text-zinc-400">
				For each integration we store the credentials you provide that are required to deliver the
				message — such as a webhook URL, a bot token, a channel or room identifier, or an email
				address. This data is associated solely with your user ID and is used server-side only;
				secrets (tokens, full webhook URLs) are never returned to the dashboard but are only marked
				as "set". Legal basis: Art. 6(1)(b) GDPR (performance of a contract). Retention: until you
				delete the integration or your account is deleted.
			</p>

			<p class="mb-1 font-medium text-zinc-300">Slack ("Add to Slack")</p>
			<p class="mb-3 text-zinc-400">
				If you connect Slack via the "Add to Slack" button, you go through Slack's OAuth flow. After
				you grant consent, Slack provides us with an incoming webhook URL and the name of the target
				channel, which we store as a Slack integration. We do not request access to your messages or
				any other workspace data. Slack's
				<a
					href="https://slack.com/trust/privacy/privacy-policy"
					class="text-indigo-400 hover:text-indigo-300"
					target="_blank"
					rel="noopener noreferrer">Privacy Policy</a
				> applies.
			</p>

			<p class="mb-1 font-medium text-zinc-300">Discord</p>
			<p class="mb-3 text-zinc-400">
				For Discord, you provide either an incoming webhook URL or a bot token together with a
				channel ID. Hoister uses these to deliver notifications to the channel you chose. Discord's
				<a
					href="https://discord.com/privacy"
					class="text-indigo-400 hover:text-indigo-300"
					target="_blank"
					rel="noopener noreferrer">Privacy Policy</a
				> applies.
			</p>

			<p class="mb-1 font-medium text-zinc-300">What data is transmitted</p>
			<p class="mb-3 text-zinc-400">
				When a deployment event occurs, Hoister sends a notification to the services you have
				configured containing the image name, image digest, project and service name, and the
				hostname of the affected container. This transmission happens on your instructions to the
				recipients you selected. The respective providers (Slack, Discord, etc.) process this
				content under their own privacy policies; for US-based providers this may involve a transfer
				to a third country. As you determine the recipient yourself, this transfer is your
				responsibility.
			</p>

			<p class="mb-1 font-medium text-zinc-300">Email delivery: Resend</p>
			<p class="text-zinc-400">
				For email notifications we use the delivery provider <span class="text-zinc-300">Resend</span
				> (Resend, Inc., USA) as a processor. The recipient email address and the content of the
				notification are passed to Resend. The transfer to the USA is based on Standard Contractual
				Clauses (Art. 46(2)(c) GDPR).
			</p>
		</section>

		<!-- 7 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">7. Cookies</h2>
			<p class="mb-3 text-zinc-400">
				This website uses <span class="text-zinc-300">strictly necessary cookies</span> only. These
				cookies are required to keep you authenticated during a session and are set without your
				separate consent (Art. 6(1)(b) and (f) GDPR). No advertising, tracking or analytics cookies
				are used.
			</p>
			<div class="overflow-x-auto rounded-lg border border-zinc-800">
				<table class="w-full text-sm">
					<thead>
						<tr class="border-b border-zinc-800 text-left text-zinc-500">
							<th class="px-4 py-3 font-medium">Cookie</th>
							<th class="px-4 py-3 font-medium">Provider</th>
							<th class="px-4 py-3 font-medium">Purpose</th>
							<th class="px-4 py-3 font-medium">Lifetime</th>
						</tr>
					</thead>
					<tbody class="text-zinc-400">
						<tr class="border-b border-zinc-800/50">
							<td class="px-4 py-3 font-mono text-xs">__session</td>
							<td class="px-4 py-3">Clerk</td>
							<td class="px-4 py-3">Authentication session</td>
							<td class="px-4 py-3">Session</td>
						</tr>
						<tr class="border-b border-zinc-800/50">
							<td class="px-4 py-3 font-mono text-xs">__client_uat</td>
							<td class="px-4 py-3">Clerk</td>
							<td class="px-4 py-3">Session verification</td>
							<td class="px-4 py-3">1 year</td>
						</tr>
						<tr>
							<td class="px-4 py-3 font-mono text-xs">slack_oauth_state</td>
							<td class="px-4 py-3">Hoister</td>
							<td class="px-4 py-3">CSRF protection during Slack connection</td>
							<td class="px-4 py-3">Short-lived</td>
						</tr>
					</tbody>
				</table>
			</div>
		</section>

		<!-- 8 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				8. Disclosure of Data
			</h2>
			<p class="text-zinc-400">
				Your personal data is not disclosed to third parties, except to our processors (Clerk for
				authentication, see Section 4; Resend for email delivery, see Section 6), to the messaging
				services you set up yourself (see Section 6), and where we are legally obliged to do so.
			</p>
		</section>

		<!-- 9 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				9. Your Rights
			</h2>
			<p class="mb-3 text-zinc-400">
				You have the following rights regarding your personal data:
			</p>
			<ul class="list-disc space-y-1 pl-5 text-zinc-400">
				<li>Right of access (Art. 15 GDPR)</li>
				<li>Right to rectification (Art. 16 GDPR)</li>
				<li>Right to erasure (Art. 17 GDPR)</li>
				<li>Right to restriction of processing (Art. 18 GDPR)</li>
				<li>Right to data portability (Art. 20 GDPR)</li>
				<li>Right to object (Art. 21 GDPR)</li>
			</ul>
			<p class="mt-3 text-zinc-400">
				To exercise your rights, please contact: <a
					href="mailto:info@hoister.io"
					class="text-indigo-400 hover:text-indigo-300">info@hoister.io</a
				>
			</p>
		</section>

		<!-- 10 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				10. Right to Lodge a Complaint with a Supervisory Authority
			</h2>
			<p class="mb-3 text-zinc-400">
				You have the right to lodge a complaint with a data protection supervisory authority about
				our processing of your personal data. The competent supervisory authority for Berlin is:
			</p>
			<div class="space-y-1 text-zinc-300">
				<p>Berliner Beauftragte für Datenschutz und Informationsfreiheit</p>
				<p class="text-zinc-400">Friedrichstr. 219, 10969 Berlin, Germany</p>
				<p class="text-zinc-400">
					Web: <a
						href="https://www.datenschutz-berlin.de"
						class="text-indigo-400 hover:text-indigo-300"
						target="_blank"
						rel="noopener noreferrer">www.datenschutz-berlin.de</a
					>
				</p>
			</div>
		</section>

		<!-- 11 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				11. Currency of This Policy
			</h2>
			<p class="text-zinc-400">
				We reserve the right to amend this privacy policy in the event of changes to the service or
				the legal situation. The current version is always available at this URL.
			</p>
		</section>
	</main>

	<footer class="border-t border-zinc-800 py-5 text-center text-xs text-zinc-600">
		Hoister — open source on GitHub ·
		<a href="/impressum" class="hover:text-zinc-400">Impressum</a> ·
		<a href="/datenschutz" class="hover:text-zinc-400">Datenschutz</a> ·
		<a href="/privacy" class="hover:text-zinc-400">Privacy (EN)</a>
	</footer>
</div>
