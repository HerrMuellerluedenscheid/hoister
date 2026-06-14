<script lang="ts">
	import SEO from '$lib/components/SEO.svelte';
</script>

<SEO
	title="Nutzungsbedingungen — Hoister"
	description="Terms of Service for Hoister."
	path="/nutzungsbedingungen"
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
		<a href="/terms" class="text-sm text-zinc-500 hover:text-zinc-300">English</a>
	</header>

	<main class="mx-auto w-full max-w-2xl flex-1 px-8 py-12">
		<h1 class="mb-2 text-3xl font-bold tracking-tight">Nutzungsbedingungen</h1>
		<p class="mb-10 text-sm text-zinc-500">Stand: Juni 2026</p>

		<!-- 1 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				1. Geltungsbereich und Zustimmung
			</h2>
			<p class="mb-3 text-zinc-400">
				Diese Nutzungsbedingungen („Bedingungen") regeln Ihre Nutzung des gehosteten Hoister-Dienstes,
				einschließlich des Web-Dashboards, der Controller-API sowie zugehöriger Anwendungen und
				Integrationen (zusammen der „Dienst"), bereitgestellt durch:
			</p>
			<div class="mb-3 space-y-1 text-zinc-300">
				<p>Vector &amp; Veneer UG (haftungsbeschränkt)</p>
				<p>Rubensstr. 108, 12157 Berlin, Deutschland</p>
				<p>
					E-Mail: <a href="mailto:info@hoister.io" class="text-indigo-400 hover:text-indigo-300"
						>info@hoister.io</a
					>
				</p>
			</div>
			<p class="text-zinc-400">
				Indem Sie ein Konto anlegen, eine Integration verbinden oder den Dienst anderweitig nutzen,
				stimmen Sie diesen Bedingungen zu. Wenn Sie nicht zustimmen, nutzen Sie den Dienst nicht.
				Nutzen Sie den Dienst im Namen einer Organisation, sichern Sie zu, dass Sie berechtigt sind,
				diese Organisation an diese Bedingungen zu binden.
			</p>
		</section>

		<!-- 2 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				2. Beschreibung des Dienstes
			</h2>
			<p class="mb-3 text-zinc-400">
				Hoister prüft regelmäßig auf neuere Container-Images, aktualisiert laufende Container und
				setzt Updates bei Fehlern zurück (Rollback). Der gehostete Dienst aggregiert die vom
				Hoister-Agent gemeldeten Status- und Deployment-Informationen und kann Benachrichtigungen
				über Deployment-Ereignisse an die von Ihnen verbundenen Messaging-Dienste (etwa Slack,
				Discord und E-Mail) zustellen.
			</p>
			<p class="text-zinc-400">
				Der Hoister-Agent kann auch standalone oder selbst gehostet ohne diesen gehosteten Dienst
				betrieben werden. Diese Betriebsmodi unterliegen der Open-Source-Lizenz des Agents und nicht
				diesen Bedingungen. Siehe
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
				3. Konten und Voraussetzungen
			</h2>
			<p class="mb-3 text-zinc-400">
				Sie müssen mindestens 18 Jahre alt und in der Lage sein, einen verbindlichen Vertrag
				abzuschließen, um den Dienst zu nutzen. Die Authentifizierung erfolgt über die Anmeldung mit
				GitHub oder Google über unseren Authentifizierungsanbieter. Sie sind für die Vertraulichkeit
				Ihrer Zugangsdaten und für alle Aktivitäten verantwortlich, die unter Ihrem Konto erfolgen.
			</p>
			<p class="text-zinc-400">
				Sie verpflichten sich, zutreffende Angaben zu machen und diese aktuell zu halten.
				Benachrichtigen Sie uns unverzüglich unter
				<a href="mailto:info@hoister.io" class="text-indigo-400 hover:text-indigo-300"
					>info@hoister.io</a
				>, wenn Sie eine unbefugte Nutzung Ihres Kontos vermuten.
			</p>
		</section>

		<!-- 4 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				4. Zulässige Nutzung
			</h2>
			<p class="mb-3 text-zinc-400">Sie verpflichten sich, Folgendes zu unterlassen:</p>
			<ul class="list-disc space-y-2 pl-5 text-zinc-400">
				<li>
					den Dienst unter Verstoß gegen geltende Gesetze oder Vorschriften oder zur Verletzung von
					Rechten Dritter zu nutzen;
				</li>
				<li>
					sich unbefugten Zugang zum Dienst, zu Daten anderer Nutzer oder zur zugrunde liegenden
					Infrastruktur zu verschaffen oder die Integrität oder Leistung des Dienstes zu stören oder
					zu beeinträchtigen;
				</li>
				<li>
					die Verwundbarkeit des Dienstes zu untersuchen, zu scannen oder zu testen, außer im Rahmen
					einer vorab mit uns vereinbarten koordinierten Offenlegung;
				</li>
				<li>
					übermäßig viele automatisierte Anfragen zu senden, die Controller-API zu überlasten oder
					Ratenbegrenzungen bzw. Nutzungsgrenzen Ihres Tarifs zu umgehen;
				</li>
				<li>
					die Benachrichtigungs-Integrationen zur Übermittlung von Spam, Schadsoftware oder
					rechtswidrigen, missbräuchlichen oder irreführenden Inhalten zu nutzen.
				</li>
			</ul>
			<p class="mt-3 text-zinc-400">
				Sie sind allein für die Konfiguration des Hoister-Agents in Ihrer Umgebung verantwortlich,
				einschließlich der Frage, welche Container er verwaltet und welche Daten er an den Dienst
				meldet.
			</p>
			<p class="mt-3 text-zinc-400">
				Der Dienst ist nicht dafür bestimmt, sensible Daten (etwa Passwörter, Tokens oder Schlüssel)
				zu empfangen, und Sie dürfen solche Daten nicht wissentlich an ihn übermitteln. Die
				schlüsselbasierte Redaction des Agents ist lediglich eine Heuristik nach bestem Bemühen; Sie
				sind dafür verantwortlich, die Übermittlung von Geheimnissen zu verhindern — indem Sie die
				Redaction-Liste erweitern (<code
					class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">redact_keywords</code
				>
				/
				<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">HOISTER_REDACT_KEYWORDS</code
				>) und keine Geheimnisse im Klartext in Umgebungsvariablen, Container-Labels oder
				Befehlszeilen-Argumenten ablegen. Siehe unsere
				<a href="/datenschutz" class="text-indigo-400 hover:text-indigo-300">Datenschutzerklärung</a>
				und
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
				5. Drittdienste und Integrationen
			</h2>
			<p class="mb-3 text-zinc-400">
				Der Dienst ermöglicht es Ihnen, Messaging-Plattformen Dritter — darunter Slack, Discord,
				Telegram, Microsoft Teams, Matrix, Mattermost, Rocket.Chat, Google Chat, Gotify, ntfy,
				Pushover, E-Mail sowie generische Webhooks — zum Empfang von Deployment-Benachrichtigungen zu
				verbinden. Wenn Sie einen solchen Dienst verbinden, ermächtigen Sie Hoister, Inhalte von
				Benachrichtigungen in Ihrem Auftrag an den von Ihnen gewählten Empfänger zu übermitteln.
			</p>
			<p class="mb-3 text-zinc-400">
				Ihre Nutzung eines Drittdienstes unterliegt den eigenen Bedingungen und
				Datenschutzbestimmungen des jeweiligen Anbieters, gegebenenfalls einschließlich der
				<a
					href="https://slack.com/terms-of-service"
					class="text-indigo-400 hover:text-indigo-300"
					target="_blank"
					rel="noopener noreferrer">Slack-Nutzungsbedingungen</a
				>
				und der
				<a
					href="https://discord.com/terms"
					class="text-indigo-400 hover:text-indigo-300"
					target="_blank"
					rel="noopener noreferrer">Discord-Nutzungsbedingungen</a
				>. Sie sind dafür verantwortlich sicherzustellen, dass Sie berechtigt sind, diese Dienste zu
				verbinden und Inhalte an die ausgewählten Kanäle zu senden. Für die Verfügbarkeit,
				Richtigkeit oder das Verhalten eines Drittdienstes übernehmen wir keine Verantwortung.
			</p>
			<p class="text-zinc-400">
				Informationen zu den im Rahmen dieser Integrationen verarbeiteten Daten finden Sie in unserer
				<a href="/datenschutz" class="text-indigo-400 hover:text-indigo-300">Datenschutzerklärung</a>.
			</p>
		</section>

		<!-- 6 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				6. Tarife, Entgelte und Abrechnung
			</h2>
			<p class="mb-3 text-zinc-400">
				Der Dienst kann mit kostenlosen und kostenpflichtigen Tarifen angeboten werden. Die
				Tarifleistungen, -grenzen und -preise werden zum Zeitpunkt des Kaufs im Dashboard
				beschrieben. Kostenpflichtige Tarife werden im Voraus wiederkehrend abgerechnet und verlängern
				sich automatisch um weitere Zeiträume, sofern sie nicht vor Ablauf des laufenden Zeitraums
				gekündigt werden.
			</p>
			<p class="text-zinc-400">
				Soweit nicht durch zwingendes Recht vorgeschrieben, sind bereits gezahlte Entgelte nicht
				erstattungsfähig. Wir können Preise oder Tarifleistungen mit angemessener Vorankündigung
				ändern; Änderungen werden zu Beginn Ihres nächsten Abrechnungszeitraums wirksam. Sie können
				jederzeit über das Dashboard mit Wirkung zum Ende des laufenden Abrechnungszeitraums kündigen.
			</p>
		</section>

		<!-- 7 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				7. Geistiges Eigentum und Open Source
			</h2>
			<p class="mb-3 text-zinc-400">
				Der Name Hoister, das Logo, das Dashboard und der gehostete Dienst stehen im Eigentum von uns
				oder unseren Lizenzgebern und sind durch geltende Gesetze zum Schutz geistigen Eigentums
				geschützt. Vorbehaltlich dieser Bedingungen gewähren wir Ihnen ein nicht ausschließliches,
				nicht übertragbares, widerrufliches Recht zum Zugriff auf und zur Nutzung des gehosteten
				Dienstes.
			</p>
			<p class="text-zinc-400">
				Der Hoister-Agent und weitere als Open Source veröffentlichte Komponenten unterliegen
				weiterhin ihren jeweiligen Open-Source-Lizenzen, die durch diese Bedingungen nicht
				eingeschränkt oder geändert werden.
			</p>
		</section>

		<!-- 8 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">8. Ihre Daten</h2>
			<p class="text-zinc-400">
				Sie behalten alle Rechte an den Daten, die Sie oder Ihre Agents an den Dienst übermitteln. Sie
				räumen uns ein beschränktes Recht ein, diese Daten ausschließlich zum Betrieb und zur
				Bereitstellung des Dienstes zu verarbeiten, wie in unserer
				<a href="/datenschutz" class="text-indigo-400 hover:text-indigo-300">Datenschutzerklärung</a>
				beschrieben. Sie sind dafür verantwortlich, dass Sie über die erforderlichen Rechte zur
				Übermittlung dieser Daten verfügen und dass die Übermittlung keine Verpflichtung gegenüber
				Dritten verletzt.
			</p>
		</section>

		<!-- 9 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				9. Verfügbarkeit des Dienstes
			</h2>
			<p class="text-zinc-400">
				Wir bemühen uns, den Dienst verfügbar zu halten, garantieren jedoch keinen
				unterbrechungs- oder fehlerfreien Betrieb. Wir können den Dienst ganz oder teilweise ändern,
				aussetzen oder einstellen, Wartungsarbeiten durchführen oder Nutzungsgrenzen festlegen. Soweit
				zumutbar, kündigen wir wesentliche Änderungen, die Ihre Nutzung des Dienstes nachteilig
				beeinträchtigen, vorab an.
			</p>
		</section>

		<!-- 10 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				10. Gewährleistungsausschluss
			</h2>
			<p class="text-zinc-400">
				Soweit nach geltendem Recht zulässig, wird der Dienst „wie besehen" und „wie verfügbar" ohne
				jegliche ausdrückliche oder stillschweigende Gewährleistung bereitgestellt, einschließlich
				stillschweigender Zusicherungen der Marktgängigkeit, der Eignung für einen bestimmten Zweck
				oder der Nichtverletzung von Rechten. Wir gewährleisten nicht, dass Container-Updates,
				Rollbacks oder Benachrichtigungen stets erfolgreich sind oder zugestellt werden. Sie sind für
				angemessene Sicherungen und Schutzmaßnahmen für Ihre Systeme verantwortlich.
			</p>
		</section>

		<!-- 11 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				11. Haftungsbeschränkung
			</h2>
			<p class="mb-3 text-zinc-400">
				Nichts in diesen Bedingungen schließt unsere Haftung aus oder beschränkt sie, soweit sie nach
				geltendem zwingendem Recht nicht ausgeschlossen oder beschränkt werden kann, einschließlich
				der Haftung für Vorsatz, grobe Fahrlässigkeit, Verletzung von Leben, Körper oder Gesundheit
				oder nach dem Produkthaftungsgesetz.
			</p>
			<p class="text-zinc-400">
				Vorbehaltlich des Vorstehenden haften wir bei leichter Fahrlässigkeit nur bei Verletzung
				einer wesentlichen Vertragspflicht (einer Pflicht, deren Erfüllung die ordnungsgemäße
				Durchführung des Vertrags überhaupt erst ermöglicht und auf deren Einhaltung Sie regelmäßig
				vertrauen dürfen); in diesen Fällen ist unsere Haftung auf den vertragstypischen,
				vorhersehbaren Schaden begrenzt. Für mittelbare Schäden, Folgeschäden, entgangenen Gewinn oder
				Datenverlust, die durch leichte Fahrlässigkeit verursacht werden, haften wir darüber hinaus
				nicht.
			</p>
		</section>

		<!-- 12 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				12. Freistellung
			</h2>
			<p class="text-zinc-400">
				Sie stellen uns von Ansprüchen Dritter frei, die aus Ihrer rechtswidrigen Nutzung des
				Dienstes, Ihrem Verstoß gegen diese Bedingungen oder Ihrer Verletzung von Rechten Dritter
				entstehen, es sei denn, der Anspruch beruht auf unserem eigenen Verstoß oder unserer
				Fahrlässigkeit.
			</p>
		</section>

		<!-- 13 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				13. Sperrung und Kündigung
			</h2>
			<p class="mb-3 text-zinc-400">
				Sie können die Nutzung des Dienstes jederzeit einstellen und Ihr Konto löschen. Wir können
				Ihren Zugang sperren oder beenden, wenn Sie wesentlich gegen diese Bedingungen verstoßen, dies
				gesetzlich erforderlich ist oder Ihre Nutzung ein Sicherheits- oder Betriebsrisiko für den
				Dienst oder andere Nutzer darstellt. Soweit angemessen, geben wir Ihnen Gelegenheit, den
				Verstoß zu beheben, und benachrichtigen Sie.
			</p>
			<p class="text-zinc-400">
				Mit Beendigung endet Ihr Recht zur Nutzung des Dienstes. Wir können Ihre Kontodaten nach einer
				angemessenen Frist löschen, vorbehaltlich gesetzlicher Aufbewahrungspflichten und unserer
				Datenschutzerklärung.
			</p>
		</section>

		<!-- 14 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				14. Änderungen dieser Bedingungen
			</h2>
			<p class="text-zinc-400">
				Wir können diese Bedingungen anpassen, wenn sich der Dienst oder die Rechtslage ändert. Über
				wesentliche Änderungen informieren wir Sie auf angemessenem Weg (z. B. per E-Mail oder per
				Hinweis im Dashboard), bevor sie wirksam werden. Ihre fortgesetzte Nutzung des Dienstes nach
				Wirksamwerden der Änderungen gilt als Annahme der geänderten Bedingungen.
			</p>
		</section>

		<!-- 15 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				15. Anwendbares Recht und Gerichtsstand
			</h2>
			<p class="text-zinc-400">
				Diese Bedingungen unterliegen dem Recht der Bundesrepublik Deutschland unter Ausschluss des
				UN-Kaufrechts (CISG). Zwingende verbraucherschützende Vorschriften des Staates, in dem Sie
				Ihren gewöhnlichen Aufenthalt haben, bleiben unberührt. Soweit gesetzlich zulässig, sind die
				Gerichte in Berlin für Streitigkeiten aus oder im Zusammenhang mit diesen Bedingungen
				zuständig.
			</p>
		</section>

		<!-- 16 -->
		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">16. Kontakt</h2>
			<p class="text-zinc-400">
				Fragen zu diesen Bedingungen richten Sie bitte an <a
					href="mailto:info@hoister.io"
					class="text-indigo-400 hover:text-indigo-300">info@hoister.io</a
				>. Angaben zum Anbieter finden Sie in unserem
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
