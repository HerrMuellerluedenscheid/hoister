<script lang="ts">
	import SEO from '$lib/components/SEO.svelte';
</script>

<SEO
	title="Terms of Service — Hoister"
	description="Terms of Service for Hoister."
	path="/terms"
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
		<a href="/nutzungsbedingungen" class="text-sm text-zinc-500 hover:text-zinc-300">Deutsch</a>
	</header>

	<main class="mx-auto w-full max-w-2xl flex-1 px-8 py-12">
		<h1 class="mb-2 text-3xl font-bold tracking-tight">Terms of Service</h1>
		<p class="mb-4 text-sm text-zinc-500">Last updated: June 2026</p>
		<p class="mb-10 text-sm text-zinc-500">
			This is an English translation provided for convenience. The legally binding version is the
			German
			<a href="/nutzungsbedingungen" class="text-indigo-400 hover:text-indigo-300"
				>Nutzungsbedingungen</a
			>.
		</p>

		<!-- 1 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				1. Scope and Acceptance
			</h2>
			<p class="mb-3 text-zinc-400">
				These Terms of Service ("Terms") govern your use of the hosted Hoister service, including the
				web dashboard, the controller API, and any related applications and integrations
				(collectively, the "Service") provided by:
			</p>
			<div class="mb-3 space-y-1 text-zinc-300">
				<p>Vector &amp; Veneer UG (haftungsbeschränkt)</p>
				<p>Rubensstr. 108, 12157 Berlin, Germany</p>
				<p>
					Email: <a
						href="mailto:info@hoister.io"
						class="text-indigo-400 hover:text-indigo-300">info@hoister.io</a
					>
				</p>
			</div>
			<p class="text-zinc-400">
				By creating an account, connecting an integration, or otherwise using the Service, you agree
				to these Terms. If you do not agree, do not use the Service. If you use the Service on behalf
				of an organization, you represent that you are authorized to bind that organization to these
				Terms.
			</p>
		</section>

		<!-- 2 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				2. Description of the Service
			</h2>
			<p class="mb-3 text-zinc-400">
				Hoister periodically checks for newer container images, applies updates to running
				containers, and rolls updates back on failure. The hosted Service aggregates status and
				deployment information reported by the Hoister agent and can deliver notifications about
				deployment events to messaging services you connect (such as Slack, Discord, and email).
			</p>
			<p class="text-zinc-400">
				The Hoister agent can also be run standalone or self-hosted without this hosted Service.
				Those modes are governed by the agent's open-source license rather than these Terms. See
				<a
					href="https://docs.hoister.io/guides/operating-modes/"
					class="text-indigo-400 hover:text-indigo-300"
					target="_blank"
					rel="noopener noreferrer">docs.hoister.io/guides/operating-modes</a
				>.
			</p>
		</section>

		<!-- 3 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				3. Accounts and Eligibility
			</h2>
			<p class="mb-3 text-zinc-400">
				You must be at least 18 years old and able to enter into a binding contract to use the
				Service. Authentication is provided through GitHub or Google sign-in via our authentication
				provider. You are responsible for maintaining the confidentiality of your account
				credentials and for all activity that occurs under your account.
			</p>
			<p class="text-zinc-400">
				You agree to provide accurate information and to keep it up to date. Notify us promptly at
				<a href="mailto:info@hoister.io" class="text-indigo-400 hover:text-indigo-300"
					>info@hoister.io</a
				> if you suspect any unauthorized use of your account.
			</p>
		</section>

		<!-- 4 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				4. Acceptable Use
			</h2>
			<p class="mb-3 text-zinc-400">You agree not to:</p>
			<ul class="list-disc space-y-2 pl-5 text-zinc-400">
				<li>
					use the Service in violation of any applicable law or regulation, or to infringe the
					rights of others;
				</li>
				<li>
					attempt to gain unauthorized access to the Service, other users' data, or the underlying
					infrastructure, or interfere with or disrupt the integrity or performance of the Service;
				</li>
				<li>
					probe, scan, or test the vulnerability of the Service except as part of a coordinated
					disclosure agreed with us in advance;
				</li>
				<li>
					send excessive automated requests, overload the controller API, or circumvent rate limits
					or usage limits of your plan;
				</li>
				<li>
					use the notification integrations to transmit spam, malware, or unlawful, abusive, or
					misleading content.
				</li>
			</ul>
			<p class="mt-3 text-zinc-400">
				You are solely responsible for the configuration of the Hoister agent in your environment,
				including which containers it manages and which data it reports to the Service.
			</p>
			<p class="mt-3 text-zinc-400">
				The Service is not intended to receive sensitive data (such as passwords, tokens, or keys),
				and you must not knowingly transmit such data to it. The agent's keyword-based redaction is
				only a best-effort heuristic; you are responsible for preventing secrets from being
				transmitted — by extending the redaction list (<code
					class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">redact_keywords</code
				>
				/
				<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">HOISTER_REDACT_KEYWORDS</code
				>) and by not placing plaintext secrets in environment variables, container labels, or
				command-line arguments. See our
				<a href="/privacy" class="text-indigo-400 hover:text-indigo-300">Privacy Policy</a> and
				<a
					href="https://docs.hoister.io/guides/monitoring/#custom-redaction-keywords"
					class="text-indigo-400 hover:text-indigo-300"
					target="_blank"
					rel="noopener noreferrer">docs.hoister.io/guides/monitoring</a
				>.
			</p>
		</section>

		<!-- 5 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				5. Third-Party Services and Integrations
			</h2>
			<p class="mb-3 text-zinc-400">
				The Service lets you connect third-party messaging platforms — including Slack, Discord,
				Telegram, Microsoft Teams, Matrix, Mattermost, Rocket.Chat, Google Chat, Gotify, ntfy,
				Pushover, email, and generic webhooks — to receive deployment notifications. When you
				connect such a service, you authorize Hoister to transmit notification content to the
				recipient you select on your behalf.
			</p>
			<p class="mb-3 text-zinc-400">
				Your use of any third-party service is subject to that provider's own terms and privacy
				policies, including, where applicable, the
				<a
					href="https://slack.com/terms-of-service"
					class="text-indigo-400 hover:text-indigo-300"
					target="_blank"
					rel="noopener noreferrer">Slack Terms of Service</a
				>
				and the
				<a
					href="https://discord.com/terms"
					class="text-indigo-400 hover:text-indigo-300"
					target="_blank"
					rel="noopener noreferrer">Discord Terms of Service</a
				>. You are responsible for ensuring you are entitled to connect those services and to send
				content to the selected channels. We are not responsible for the availability, accuracy, or
				conduct of any third-party service.
			</p>
			<p class="text-zinc-400">
				Information about the data processed in connection with these integrations is described in
				our
				<a href="/privacy" class="text-indigo-400 hover:text-indigo-300">Privacy Policy</a>.
			</p>
		</section>

		<!-- 6 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				6. Plans, Fees, and Billing
			</h2>
			<p class="mb-3 text-zinc-400">
				The Service may be offered with free and paid plans. Plan features, limits, and prices are
				described in the dashboard at the time of purchase. Paid plans are billed in advance on a
				recurring basis and renew automatically for successive periods unless cancelled before the
				end of the current period.
			</p>
			<p class="text-zinc-400">
				Unless required by mandatory law, fees already paid are non-refundable. We may change prices
				or plan features with reasonable advance notice; changes take effect at the start of your
				next billing period. You can cancel at any time, effective at the end of the current billing
				period, through the dashboard.
			</p>
		</section>

		<!-- 7 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				7. Intellectual Property and Open Source
			</h2>
			<p class="mb-3 text-zinc-400">
				The Hoister name, logo, dashboard, and the hosted Service are owned by us or our licensors
				and are protected by applicable intellectual-property laws. Subject to these Terms, we grant
				you a non-exclusive, non-transferable, revocable right to access and use the hosted Service.
			</p>
			<p class="text-zinc-400">
				The Hoister agent and other components published as open source remain governed by their
				respective open-source licenses, which are not limited or modified by these Terms.
			</p>
		</section>

		<!-- 8 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				8. Your Data
			</h2>
			<p class="text-zinc-400">
				You retain all rights to the data you or your agents submit to the Service. You grant us a
				limited right to process that data solely to operate and provide the Service, as described in
				our <a href="/privacy" class="text-indigo-400 hover:text-indigo-300">Privacy Policy</a>. You
				are responsible for ensuring you have the necessary rights to submit that data and that doing
				so does not violate any obligation you owe to a third party.
			</p>
		</section>

		<!-- 9 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				9. Service Availability
			</h2>
			<p class="text-zinc-400">
				We strive to keep the Service available but do not guarantee uninterrupted or error-free
				operation. We may modify, suspend, or discontinue all or part of the Service, perform
				maintenance, or impose usage limits at any time. Where reasonably practicable, we will give
				advance notice of material changes that adversely affect your use of the Service.
			</p>
		</section>

		<!-- 10 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				10. Disclaimer of Warranties
			</h2>
			<p class="text-zinc-400">
				To the extent permitted by applicable law, the Service is provided "as is" and "as available"
				without warranties of any kind, whether express or implied, including any implied warranties
				of merchantability, fitness for a particular purpose, or non-infringement. We do not warrant
				that container updates, rollbacks, or notifications will always succeed or be delivered. You
				are responsible for maintaining appropriate backups and safeguards for your systems.
			</p>
		</section>

		<!-- 11 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				11. Limitation of Liability
			</h2>
			<p class="mb-3 text-zinc-400">
				Nothing in these Terms excludes or limits our liability where it cannot be excluded or
				limited under applicable mandatory law, including liability for intent, gross negligence,
				injury to life, body, or health, or under the German Product Liability Act
				(Produkthaftungsgesetz).
			</p>
			<p class="text-zinc-400">
				Subject to the foregoing, we are liable for slight negligence only in the event of a breach
				of a material contractual obligation (an obligation whose fulfilment is essential to the
				proper performance of the contract and on whose observance you may regularly rely), and in
				such cases our liability is limited to the foreseeable damage typical for this type of
				contract. We are not liable for indirect or consequential damages, loss of profits, or loss
				of data caused by slight negligence beyond this scope.
			</p>
		</section>

		<!-- 12 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				12. Indemnification
			</h2>
			<p class="text-zinc-400">
				You will indemnify and hold us harmless from third-party claims arising out of your unlawful
				use of the Service, your breach of these Terms, or your infringement of the rights of a
				third party, except to the extent the claim results from our own breach or negligence.
			</p>
		</section>

		<!-- 13 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				13. Suspension and Termination
			</h2>
			<p class="mb-3 text-zinc-400">
				You may stop using the Service and delete your account at any time. We may suspend or
				terminate your access if you materially breach these Terms, if required by law, or if your
				use poses a security or operational risk to the Service or other users. Where appropriate, we
				will give you notice and an opportunity to remedy the breach.
			</p>
			<p class="text-zinc-400">
				Upon termination, your right to use the Service ends. We may delete your account data after a
				reasonable period, subject to any statutory retention obligations and our Privacy Policy.
			</p>
		</section>

		<!-- 14 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				14. Changes to These Terms
			</h2>
			<p class="text-zinc-400">
				We may amend these Terms when the Service or the legal situation changes. We will notify you
				of material changes by a reasonable means (e.g. email or a notice in the dashboard) before
				they take effect. Your continued use of the Service after the changes take effect constitutes
				acceptance of the revised Terms.
			</p>
		</section>

		<!-- 15 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				15. Governing Law and Jurisdiction
			</h2>
			<p class="text-zinc-400">
				These Terms are governed by the laws of the Federal Republic of Germany, excluding the UN
				Convention on Contracts for the International Sale of Goods (CISG). Mandatory consumer
				protection provisions of the country in which you have your habitual residence remain
				unaffected. To the extent permitted by law, the courts of Berlin, Germany have jurisdiction
				over disputes arising out of or in connection with these Terms.
			</p>
		</section>

		<!-- 16 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				16. Contact
			</h2>
			<p class="text-zinc-400">
				Questions about these Terms can be directed to <a
					href="mailto:info@hoister.io"
					class="text-indigo-400 hover:text-indigo-300">info@hoister.io</a
				>. Provider identification is set out in our
				<a href="/impressum" class="text-indigo-400 hover:text-indigo-300">Impressum</a>.
			</p>
		</section>
	</main>

	<footer class="border-t border-zinc-800 py-5 text-center text-xs text-zinc-600">
		Hoister — open source on GitHub ·
		<a href="/impressum" class="hover:text-zinc-400">Impressum</a> ·
		<a href="/datenschutz" class="hover:text-zinc-400">Datenschutz</a> ·
		<a href="/nutzungsbedingungen" class="hover:text-zinc-400">Nutzungsbedingungen</a> ·
		<a href="/privacy" class="hover:text-zinc-400">Privacy (EN)</a> ·
		<a href="/terms" class="hover:text-zinc-400">Terms (EN)</a>
	</footer>
</div>
