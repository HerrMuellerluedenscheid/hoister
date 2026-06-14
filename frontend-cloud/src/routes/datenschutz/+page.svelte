<script lang="ts">
	import SEO from '$lib/components/SEO.svelte';
</script>

<SEO
	title="Datenschutz — Hoister"
	description="Privacy policy for Hoister."
	path="/datenschutz"
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
		<a href="/privacy" class="text-sm text-zinc-500 hover:text-zinc-300">English</a>
	</header>

	<main class="mx-auto w-full max-w-2xl flex-1 px-8 py-12">
		<h1 class="mb-2 text-3xl font-bold tracking-tight">Datenschutzerklärung</h1>
		<p class="mb-10 text-sm text-zinc-500">Stand: Juni 2026</p>

		<!-- 1 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				1. Verantwortlicher
			</h2>
			<div class="space-y-1 text-zinc-300">
				<p>Vector &amp; Veneer UG (haftungsbeschränkt)</p>
				<p>Rubensstr. 108, 12157 Berlin</p>
				<p>
					E-Mail: <a
						href="mailto:info@vectorandveneer.com"
						class="text-indigo-400 hover:text-indigo-300">info@vectorandveneer.com</a
					>
				</p>
			</div>
		</section>

		<!-- 2 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				2. Welche Daten wir verarbeiten
			</h2>
			<p class="mb-3 text-zinc-400">
				Beim Betrieb dieses Dienstes fallen folgende personenbezogene Daten an:
			</p>
			<ul class="list-disc space-y-2 pl-5 text-zinc-400">
				<li>
					<span class="text-zinc-300">Server-Logdaten</span> – IP-Adresse, Datum und Uhrzeit der Anfrage,
					aufgerufene URL, HTTP-Statuscode, übertragene Datenmenge, Referrer, Browser- und Betriebssystem-Kennung.
					Diese Daten werden automatisch beim Aufruf der Website erfasst.
				</li>
				<li>
					<span class="text-zinc-300">Kontodaten</span> – E-Mail-Adresse sowie OAuth-Profilinformationen
					(Name, Profilbild), die Sie beim Anmelden über GitHub oder Google bereitstellen.
				</li>
				<li>
					<span class="text-zinc-300">Authentifizierungs-Sitzungsdaten</span> – verschlüsselte Session-Tokens,
					die in Cookies gespeichert werden, um Sie während einer Sitzung eingeloggt zu halten.
				</li>
				<li>
					<span class="text-zinc-300">Konfiguration von Benachrichtigungs-Integrationen</span> – die
					Zugangsdaten, die Sie zum Verbinden eines Messaging-Dienstes angeben (z. B. Slack-Webhook-URLs,
					Discord-Bot-Tokens oder -Webhook-URLs, Kanal-Bezeichner, E-Mail-Empfänger). Siehe Abschnitt 6.
				</li>
			</ul>
		</section>

		<!-- 3 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				3. Zwecke und Rechtsgrundlagen der Verarbeitung
			</h2>
			<div class="space-y-4 text-zinc-400">
				<div>
					<p class="mb-1 font-medium text-zinc-300">Server-Logdaten</p>
					<p>
						Zweck: Sicherstellung des technisch fehlerfreien Betriebs und Erkennung von Angriffen.
						Rechtsgrundlage: Art. 6 Abs. 1 lit. f DSGVO (berechtigtes Interesse). Speicherdauer:
						maximal 14 Tage, danach automatische Löschung.
					</p>
				</div>
				<div>
					<p class="mb-1 font-medium text-zinc-300">Kontodaten &amp; Authentifizierung</p>
					<p>
						Zweck: Bereitstellung des Nutzerkontos und der Dienst-Funktionen (z. B. Dashboard,
						Deployment-Verwaltung). Rechtsgrundlage: Art. 6 Abs. 1 lit. b DSGVO (Vertragserfüllung
						bzw. vorvertragliche Maßnahmen). Speicherdauer: bis zur Löschung des Kontos.
					</p>
				</div>
			</div>
		</section>

		<!-- 4 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				4. Authentifizierungsdienst: Clerk
			</h2>
			<p class="mb-3 text-zinc-400">
				Für die Nutzerverwaltung und Authentifizierung setzen wir den Dienst <span
					class="text-zinc-300">Clerk</span
				> ein (Clerk, Inc., 2 Park Avenue, Floor 20, New York, NY 10016, USA).
			</p>
			<p class="mb-3 text-zinc-400">
				Clerk verarbeitet Ihren Namen, Ihre E-Mail-Adresse und Authentifizierungstokens auf unsere
				Weisung hin als Auftragsverarbeiter gemäß Art. 28 DSGVO. Ein Datenverarbeitungsvertrag (DPA)
				ist abgeschlossen.
			</p>
			<p class="text-zinc-400">
				Da Clerk seinen Sitz in den USA hat, werden Daten in ein Drittland übermittelt. Die
				Übermittlung erfolgt auf Grundlage von Standardvertragsklauseln (SCCs) der EU-Kommission
				gemäß Art. 46 Abs. 2 lit. c DSGVO. Weitere Informationen: <a
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
				5. Vom Hoister-Agent übermittelte Telemetrie
			</h2>
			<p class="mb-3 text-zinc-400">
				Wenn Sie den Hoister-Agent (<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs"
					>hoister/hoister</code
				>) so konfigurieren, dass er sich mit dieser gehosteten Controller-Instanz verbindet (<code
					class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs"
					>HOISTER_CONTROLLER_URL=https://api.hoister.io</code
				>), überträgt der Agent in regelmäßigen Abständen folgende Daten zu Ihrem Konto:
			</p>
			<ul class="list-disc space-y-2 pl-5 text-zinc-400">
				<li>
					<span class="text-zinc-300">Container-Metadaten</span> – Image-Namen (inklusive privater Registry-Pfade),
					Container-Labels, Mount-Pfade auf dem Host, IP-Adressen aus den von Docker verwalteten Netzwerken,
					Befehlszeilen-Argumente sowie Konfigurations- und Statusfelder, die der Docker-Inspect-Endpunkt
					liefert.
				</li>
				<li>
					<span class="text-zinc-300">Umgebungsvariablen-Schlüssel</span> – Der Agent überträgt die
					<em>Namen</em> Ihrer Umgebungsvariablen (z. B.
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">INTERNAL_API_HOST</code>).
					<span class="text-zinc-300">Werte</span> von Variablen, deren Schlüssel auf eine sensible
					Bezeichnung schließen lassen (<code
						class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">password</code
					>, <code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">token</code>,
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">secret</code>,
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">key</code>,
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">webhook</code> und
					ähnliches) werden vor der Übermittlung im Agent durch
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">***REDACTED***</code>
					ersetzt. Diese Erkennungsliste lässt sich um eigene Schlüssel erweitern (siehe Hinweis
					unten).
				</li>
				<li>
					<span class="text-zinc-300">Deployment-Ereignisse</span> – Erfolgreiche oder fehlgeschlagene
					Container-Updates samt Image-Digest, Hostname, Projekt- und Service-Name.
				</li>
				<li>
					<span class="text-zinc-300">Container-Log-Auszüge (nur opt-in)</span> – Standardmäßig
					<strong>deaktiviert</strong>. Wird der Agent mit
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs"
						>HOISTER_REPORT_LOGS=true</code
					>
					gestartet und befindet sich ein Container im Zustand <em>restarting</em>,
					<em>exited</em> oder <em>dead</em>, überträgt der Agent die letzten ~50 Zeilen
					Container-Log (maximal 16 KB). Vor der Übermittlung werden Werte sensibler
					Umgebungsvariablen aus dem Log entfernt. Trotz dieser Heuristik können Logs sensible Daten
					enthalten — aktivieren Sie diese Option nur, wenn Sie das akzeptieren.
				</li>
			</ul>
			<div class="mt-4 rounded-lg border border-zinc-800 bg-zinc-900/40 p-4 text-zinc-400">
				<p class="mb-2 font-medium text-zinc-300">
					Ihre Verantwortung für sensible Daten
				</p>
				<p>
					Wir möchten ausdrücklich <strong>keine</strong> sensiblen Daten (Passwörter, Tokens,
					Schlüssel o. ä.) erhalten. Die schlüsselbasierte Redaction ist eine Heuristik und kann
					nicht jede geheime Variable zuverlässig erkennen. Sie sind dafür verantwortlich
					sicherzustellen, dass keine Geheimnisse an den Dienst übermittelt werden: Erweitern Sie
					die Redaction-Liste über
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">redact_keywords</code>
					(in der
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">hoister.toml</code>)
					bzw. die kommagetrennte Umgebungsvariable
					<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs"
						>HOISTER_REDACT_KEYWORDS</code
					>
					um Ihre eigenen Schlüssel und vermeiden Sie es, Geheimnisse im Klartext in
					Umgebungsvariablen, Container-Labels oder Befehlszeilen-Argumenten abzulegen. So gelangen
					sensible Werte gar nicht erst auf unsere Server. Details:
					<a
						href="https://docs.hoister.io/guides/monitoring/#custom-redaction-keywords"
						class="text-indigo-400 hover:text-indigo-300"
						target="_blank"
						rel="noopener noreferrer">docs.hoister.io/guides/monitoring</a
					>.
				</p>
			</div>
			<p class="mt-4 text-zinc-400">
				<span class="text-zinc-300">Zweck:</span> Anzeige des Status Ihrer Container und
				Deployment-Historie in Ihrem Dashboard.<br />
				<span class="text-zinc-300">Rechtsgrundlage:</span> Art. 6 Abs. 1 lit. b DSGVO
				(Vertragserfüllung).<br />
				<span class="text-zinc-300">Speicherort:</span> EU (Server in Deutschland).<br />
				<span class="text-zinc-300">Speicherdauer:</span> Container-Zustände werden im Speicher
				gehalten und sind nach einem Neustart der Controller-Instanz verloren. Deployment-Datensätze
				bleiben bis zur Löschung Ihres Kontos erhalten.<br />
				<span class="text-zinc-300">Mandantentrennung:</span> Alle übermittelten Daten werden ausschließlich
				unter Ihrer Nutzer-ID gespeichert und sind anderen Nutzern technisch nicht zugänglich.
			</p>
			<p class="mt-3 text-zinc-400">
				Sie können dieser Verarbeitung jederzeit widersprechen, indem Sie den Agent ohne
				<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs"
					>HOISTER_CONTROLLER_TOKEN</code
				>
				betreiben (Standalone-Modus, keine Telemetrie) oder eine eigene Controller-Instanz aufsetzen (Self-hosted-Modus).
				Details:
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
				6. Benachrichtigungs-Integrationen (Slack, Discord, E-Mail u. a.)
			</h2>
			<p class="mb-3 text-zinc-400">
				Optional können Sie einen oder mehrere Messaging-Dienste mit Ihrem Konto verbinden, damit
				Hoister Sie über Deployment-Ereignisse (verfügbare Updates, erfolgreiche Updates,
				Rollbacks) benachrichtigt. Unterstützt werden u. a. <span class="text-zinc-300"
					>Slack, Discord, Telegram, Microsoft Teams, Matrix, Mattermost, Rocket.Chat, Google
					Chat, Gotify, ntfy, Pushover, E-Mail sowie generische Webhooks</span
				>. Diese Funktion ist optional und nur aktiv, wenn Sie eine Integration einrichten.
			</p>

			<p class="mb-1 font-medium text-zinc-300">Was wir speichern</p>
			<p class="mb-3 text-zinc-400">
				Pro Integration speichern wir die von Ihnen angegebenen Zugangsdaten, die zum Zustellen der
				Nachricht erforderlich sind – etwa eine Webhook-URL, einen Bot-Token, eine Kanal- oder
				Raum-Kennung oder eine E-Mail-Adresse. Diese Daten sind ausschließlich Ihrer Nutzer-ID
				zugeordnet und werden nur serverseitig verwendet; im Dashboard werden Geheimnisse (Tokens,
				vollständige Webhook-URLs) nicht zurückgegeben, sondern nur als „gesetzt“ markiert.
				Rechtsgrundlage: Art. 6 Abs. 1 lit. b DSGVO (Vertragserfüllung). Speicherdauer: bis Sie die
				Integration löschen bzw. bis zur Löschung Ihres Kontos.
			</p>

			<p class="mb-1 font-medium text-zinc-300">Slack („Add to Slack“)</p>
			<p class="mb-3 text-zinc-400">
				Verbinden Sie Slack über den „Add to Slack“-Button, durchlaufen Sie den OAuth-Ablauf von
				Slack. Nach Ihrer Zustimmung erhalten wir von Slack eine eingehende Webhook-URL und den
				Namen des Zielkanals und speichern diese als Slack-Integration. Wir fragen keinen Zugriff
				auf Ihre Nachrichten oder weitere Workspace-Daten an. Es gelten die
				<a
					href="https://slack.com/trust/privacy/privacy-policy"
					class="text-indigo-400 hover:text-indigo-300"
					target="_blank"
					rel="noopener noreferrer">Datenschutzbestimmungen von Slack</a
				>.
			</p>

			<p class="mb-1 font-medium text-zinc-300">Discord</p>
			<p class="mb-3 text-zinc-400">
				Für Discord geben Sie entweder eine eingehende Webhook-URL oder einen Bot-Token samt
				Kanal-ID an. Damit stellt Hoister die Benachrichtigungen im von Ihnen gewählten Kanal zu.
				Es gelten die
				<a
					href="https://discord.com/privacy"
					class="text-indigo-400 hover:text-indigo-300"
					target="_blank"
					rel="noopener noreferrer">Datenschutzbestimmungen von Discord</a
				>.
			</p>

			<p class="mb-1 font-medium text-zinc-300">Welche Daten übermittelt werden</p>
			<p class="mb-3 text-zinc-400">
				Tritt ein Deployment-Ereignis ein, sendet Hoister an die von Ihnen konfigurierten Dienste
				eine Benachrichtigung mit Image-Name, Image-Digest, Projekt- und Service-Name sowie
				Hostname des betroffenen Containers. Diese Übermittlung erfolgt auf Ihre Weisung an die von
				Ihnen ausgewählten Empfänger. Die jeweiligen Anbieter (Slack, Discord usw.) verarbeiten
				diese Inhalte nach ihren eigenen Datenschutzbestimmungen; bei US-Anbietern kann dies eine
				Übermittlung in ein Drittland bedeuten. Da Sie den Empfänger selbst bestimmen, liegt diese
				Übermittlung in Ihrer Verantwortung.
			</p>

			<p class="mb-1 font-medium text-zinc-300">E-Mail-Versand: Resend</p>
			<p class="text-zinc-400">
				Für E-Mail-Benachrichtigungen nutzen wir den Versanddienstleister <span class="text-zinc-300"
					>Resend</span
				> (Resend, Inc., USA) als Auftragsverarbeiter. An Resend werden die Empfänger-E-Mail-Adresse
				und der Inhalt der Benachrichtigung übergeben. Die Übermittlung in die USA erfolgt auf
				Grundlage von Standardvertragsklauseln (Art. 46 Abs. 2 lit. c DSGVO).
			</p>
		</section>

		<!-- 7 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">7. Cookies</h2>
			<p class="mb-3 text-zinc-400">
				Diese Website setzt ausschließlich <span class="text-zinc-300"
					>technisch notwendige Cookies</span
				>. Diese Cookies sind erforderlich, um Sie während einer Sitzung authentifiziert zu halten,
				und werden ohne Ihre gesonderte Einwilligung gesetzt (Art. 6 Abs. 1 lit. b und lit. f
				DSGVO). Es werden keine Werbe-, Tracking- oder Analyse-Cookies verwendet.
			</p>
			<div class="overflow-x-auto rounded-lg border border-zinc-800">
				<table class="w-full text-sm">
					<thead>
						<tr class="border-b border-zinc-800 text-left text-zinc-500">
							<th class="px-4 py-3 font-medium">Cookie</th>
							<th class="px-4 py-3 font-medium">Anbieter</th>
							<th class="px-4 py-3 font-medium">Zweck</th>
							<th class="px-4 py-3 font-medium">Laufzeit</th>
						</tr>
					</thead>
					<tbody class="text-zinc-400">
						<tr class="border-b border-zinc-800/50">
							<td class="px-4 py-3 font-mono text-xs">__session</td>
							<td class="px-4 py-3">Clerk</td>
							<td class="px-4 py-3">Authentifizierungs-Session</td>
							<td class="px-4 py-3">Sitzung</td>
						</tr>
						<tr class="border-b border-zinc-800/50">
							<td class="px-4 py-3 font-mono text-xs">__client_uat</td>
							<td class="px-4 py-3">Clerk</td>
							<td class="px-4 py-3">Sitzungs-Verifizierung</td>
							<td class="px-4 py-3">1 Jahr</td>
						</tr>
						<tr>
							<td class="px-4 py-3 font-mono text-xs">slack_oauth_state</td>
							<td class="px-4 py-3">Hoister</td>
							<td class="px-4 py-3">CSRF-Schutz während der Slack-Verbindung</td>
							<td class="px-4 py-3">kurzlebig</td>
						</tr>
					</tbody>
				</table>
			</div>
		</section>

		<!-- 8 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				8. Weitergabe von Daten
			</h2>
			<p class="text-zinc-400">
				Eine Weitergabe Ihrer personenbezogenen Daten an Dritte findet nicht statt, außer an unsere
				Auftragsverarbeiter (Clerk für Authentifizierung, siehe Abschnitt 4; Resend für
				E-Mail-Versand, siehe Abschnitt 6), an die von Ihnen selbst eingerichteten
				Messaging-Dienste (siehe Abschnitt 6) sowie soweit wir gesetzlich dazu verpflichtet sind.
			</p>
		</section>

		<!-- 9 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				9. Ihre Rechte
			</h2>
			<p class="mb-3 text-zinc-400">
				Sie haben gegenüber uns folgende Rechte hinsichtlich Ihrer personenbezogenen Daten:
			</p>
			<ul class="list-disc space-y-1 pl-5 text-zinc-400">
				<li>Recht auf Auskunft (Art. 15 DSGVO)</li>
				<li>Recht auf Berichtigung (Art. 16 DSGVO)</li>
				<li>Recht auf Löschung (Art. 17 DSGVO)</li>
				<li>Recht auf Einschränkung der Verarbeitung (Art. 18 DSGVO)</li>
				<li>Recht auf Datenübertragbarkeit (Art. 20 DSGVO)</li>
				<li>Widerspruchsrecht (Art. 21 DSGVO)</li>
			</ul>
			<p class="mt-3 text-zinc-400">
				Zur Ausübung Ihrer Rechte wenden Sie sich bitte an: <a
					href="mailto:info@hoister.io"
					class="text-indigo-400 hover:text-indigo-300">info@hoister.io</a
				>
			</p>
		</section>

		<!-- 10 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				10. Beschwerderecht bei der Aufsichtsbehörde
			</h2>
			<p class="mb-3 text-zinc-400">
				Sie haben das Recht, sich bei einer Datenschutz-Aufsichtsbehörde über die Verarbeitung Ihrer
				personenbezogenen Daten durch uns zu beschweren. Die zuständige Aufsichtsbehörde für Berlin
				ist:
			</p>
			<div class="space-y-1 text-zinc-300">
				<p>Berliner Beauftragte für Datenschutz und Informationsfreiheit</p>
				<p class="text-zinc-400">Friedrichstr. 219, 10969 Berlin</p>
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
				11. Aktualität dieser Erklärung
			</h2>
			<p class="text-zinc-400">
				Wir behalten uns vor, diese Datenschutzerklärung bei Änderungen des Dienstes oder der
				Rechtslage anzupassen. Die jeweils aktuelle Fassung ist stets unter dieser URL abrufbar.
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
