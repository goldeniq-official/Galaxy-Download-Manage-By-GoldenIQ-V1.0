- **Main finding:** proxy support conflicts with the current W-02 guarantee. An ordinary remote proxy can resolve a hostname differently from GDM. Local DNS validation does not prevent that. Resolve this design conflict before implementing row 11.
- **M-08 is undefined in the supplied ledger.** Give it an acceptance criterion before claiming coverage.

**A. Build order and highest risks**

1. **Establish acceptance tests and storage headroom.** Keep existing downloads/checkpoints working; add migration fixtures and bounded test downloads. CI has never run, so record local commands, results, build revision, and environment explicitly.
2. **Resolve networking architecture:** proxy/W-02 policy, connection-bound authentication, per-server limits. These affect sync, refresh, video, and every download.
3. **Implement persistent queues and scheduler**, then quotas, sync, wake tasks, and finish actions. Finish actions depend on trustworthy queue completion semantics.
4. **Implement constrained ingress:** CLI, import/batch, clipboard. All should converge on validated engine requests with source-specific capabilities.
5. **Implement refresh**, preserving old partial files and validators.
6. **Implement hooks** through their separate trusted configuration path.
7. **Implement components/video**, after the networking and process-execution boundaries are established.
8. **Run failure injection early and again before beta.** An ADR-1 failure could require architectural work; do not discover it after polishing the UI.
9. **Finish UI conveniences, FTP/FTPS, packaging**, then speed gate and the two-week beta.

- Highest risks: proxy DNS bypass; NTLM connection reuse; executable/component trust; batch-script argument handling; refresh corrupting partial downloads; shutdown racing new work.
- Define whether FTP/FTPS and the other ledger items are v1.0 blockers now. Otherwise “IDM parity” has no stable completion boundary.

**B. Concrete designs and pitfalls**

- **Queue scheduler / DST**
  - Persist an explicit time-zone ID, local time, recurrence rule, rule revision, and enabled state. Define whether changing Windows time zone changes an existing schedule.
  - Convert each local occurrence using the zone rules: nonexistent → first valid instant; ambiguous → one selected occurrence.
  - Atomically claim occurrences in SQLite with a unique key such as `(queue, rule_revision, local_date, occurrence_slot)`. Date alone breaks periodic schedules with multiple runs per day.
  - Persist claims before dispatch; recover unfinished dispatches idempotently. Explicitly define missed-run behavior after sleep, app downtime, or clock changes.
  - Use wall time to determine scheduled occurrences and monotonic time for countdowns. Define overnight windows and whether stop time pauses active files.

- **Finish actions / Q-02**
  - Model `completed`, `paused`, `cancelled`, `failed`, and `quota_blocked` separately. “No runnable jobs” does not mean finished.
  - Start one countdown for a queue completion generation. New work, manual stop, retry scheduling, or settings changes invalidate it.
  - At zero, serialize the final check with queue mutations and consume that completion generation once. Persist enough state to prevent repeated actions after restart.
  - Use Windows power APIs; force termination requires its own explicit setting. Report denied power operations.

- **Hooks / Q-01**
  - A dedicated IPC name alone is insufficient: scope its Tauri permission to the Settings window and validate its caller. The pipe protocol and import DTOs must have no hook capability.
  - Persist an absolute executable path and an argv array. Expand placeholders directly into individual arguments; never construct a command line by concatenation.
  - Replacing the executable after configuration remains possible. Describe hooks as user-trusted execution and show the exact program and arguments.
  - For `.bat`/`.cmd`, apply D15’s rejection rule to **final expanded arguments**, including filenames. A downloaded filename can contain shell metacharacters.
  - Rust ≥1.77.2 is a minimum requirement, not sufficient evidence of safety. Windows batch execution still involves command-interpreter semantics. Reject CR/LF and test empty arguments, whitespace, trailing backslashes, Unicode, and quoting against actual scripts.
  - Persist a completion-event identifier to prevent accidental duplicate execution. Exactly-once external execution across a crash is generally unavailable; specify the recovery policy.

- **CLI / named pipe**
  - Parse CLI mode before constructing Tauri. A secondary invocation authenticates the pipe server, submits one typed request, prints the result, and exits.
  - If no primary exists, start the primary and retry through the pipe. Coordinate startup with a per-user mutex; handle two simultaneous launches. Keep single-instance activation for ordinary GUI launches, not as the command transport.
  - Authenticate **both ends**: SID-restricted DACL, reject remote pipe clients, verify peer process/token identity, and prevent pipe-name squatting. Define same-user versus same-logon-session behavior.
  - Same-user authentication does not establish trusted intent: any process under that account may submit permitted requests. Keep hooks/settings outside the protocol.
  - Bound message size, deadlines, arguments, and response size; add request IDs for retry deduplication.
  - `/p`: require an existing directory, open a directory handle, resolve its final identity, and compare against resolved protected directories using boundary-aware ancestry checks. Cover UNC paths, device syntax, short names, case, junctions, and volume aliases.
  - A resolved string alone leaves a replacement race. Bind destination creation to the validated directory identity or revalidate the opened output handle before writing.
  - Specify every flag’s behavior, especially combinations of `/a`, `/s`, `/n`, and `/q`; return meaningful exit codes.

- **Clipboard**
  - Subscribe to clipboard changes; avoid polling. When enabled, check the exclusion format **before reading text**.
  - Bound reads; parse URLs structurally; match the decoded URL path extension, not query-string suffixes. Define multiple-URL handling.
  - Deduplicate clipboard sequence numbers and self-written content. Never log clipboard contents or parse failures containing text.
  - Show persistent tray state. Enabling monitoring must not silently bypass normal download confirmation policy.

- **Credential Manager**
  - Store opaque credential references in DB/settings; keep passwords out of IPC responses, logs, exports, crash diagnostics, and URLs.
  - Scope targets by application, credential purpose, scheme, host, port, and path boundary. Distinguish proxy credentials from origin credentials.
  - Do not send credentials across redirects or preemptively outside their scope. Match `/private/` distinctly from `/private-other/`.
  - Handle missing/deleted credentials, rotation, and uninstall cleanup. Credential Manager protects storage; it does not isolate secrets from other code running as the same user.

- **System proxy / PAC / W-02**
  - Use a Windows proxy-resolution layer with bounded PAC evaluation and explicit handling of proxy lists, fallback, bypass, and network changes. `Proxy::custom` alone is not a PAC engine.
  - WPAD/PAC output is routing input, never permission to bypass destination checks.
  - Checking local DNS and then sending `CONNECT hostname:443` leaves proxy-side DNS unconstrained.
  - Defensible choices: refuse untrusted remote-resolution routes under strict W-02; require a trusted enforcing proxy; or connect through the proxy to a validated **IP literal**, while preserving origin Host/SNI and certificate validation.
  - The IP-literal approach needs compatibility testing and custom transport work, particularly for HTTP forwarding and authentication. Never silently downgrade to hostname resolution.
  - Keep proxy-endpoint policy distinct from destination policy: a legitimate enterprise proxy may itself be private.

- **SSPI / NTLM / Negotiate**
  - Treat this as a transport capability decision, not an auth-header helper.
  - NTLM needs an authenticated connection retained through handshake and subsequent requests. Key pools by proxy/origin, identity, and authentication context; prevent cross-user reuse.
  - Handle 401 and 407 separately, drain challenge bodies correctly, bound handshake loops, and support safe replay only.
  - Start with HTTP/1.1 for connection-bound authentication. Verify that the chosen transport exposes connection ownership; ordinary reqwest request hooks are insufficient evidence.
  - Default Windows credentials require explicit scope. Test SPNs, Kerberos fallback, channel binding, proxy tunnels, reconnects, and connection closure between challenges.

- **yt-dlp policy proxy**
  - Bind loopback only; require a per-run secret on every connection. Redact it from diagnostics and acknowledge same-user processes are outside a strong isolation boundary.
  - Validate CONNECT authority and port, resolve with W-02, then connect to the exact validated address. Revalidate every new connection; reject ambiguous authorities and unsupported proxy methods.
  - Bound concurrent tunnels, bytes, headers, and idle time. Tie lifetime to the extraction process.
  - **A CONNECT tunnel cannot inspect HTTPS redirects or response content.** It can enforce destination policy on subsequent connections, not yt-dlp behavior inside TLS.
  - Proxy configuration is not a sandbox: verify extractors cannot make direct requests or invoke external downloaders. If direct egress cannot be constrained, document the ADR-6 enforcement gap.
  - Validate extraction JSON as hostile input. GDM must independently validate every manifest, stream, segment, key, and redirect it fetches.
  - Verify that the executable actually launched is the verified file; test the handle-sharing arrangement against Windows process creation. Also protect the component directory and signed-manifest version policy.

**C. Evidence this machine cannot supply**

- **Speed parity:** blocked by the stated NVMe/free-space requirement. Report “not measured—hardware prerequisite unmet.”
- **Beta outcome:** requires ten actual testers, including five Khmer speakers, and two weeks. Report recruitment and results; never substitute developer impressions.
- **Store publication:** requires owner account, identity, and any applicable fees.
- **Enterprise auth:** local mocks can test protocol handling; real Kerberos/domain, enterprise PAC, and NTLM interoperability require suitable infrastructure or accounts.
- **Wake/power:** configuration can be inspected locally; actual wake requires supported firmware/power states and observed testing. Hibernate may be unavailable.
- **ADR-1:** testable locally, subject to disk headroom. Kill all relevant WebView2 children and separately kill `gdm.exe`; inspect downloaded bytes and recovered state.
- **CI:** report “not run,” separately from passing local tests.
- Use **implemented / locally verified / externally verified / blocked**, with evidence and prerequisites for each card.

**D. Must-have acceptance tests**

- **Q-01:** unauthorized callers and ingress rejected; final-placeholder metacharacters rejected for scripts; argument round-trip fixtures; no shell for executables; restart/duplicate completion behavior.
- **Q-02:** manual stop, pending retry, failed job, quota pause, new job during countdown, cancel at deadline, restart, and denied power action. Use an injected power-action recorder.
- **M-01:** option-injection URLs, hostile config/plugin files, external downloader attempts, direct-egress attempts, redirects to blocked destinations, subprocess cleanup.
- **M-02:** its definition is missing; if this is component integrity, test bad signature/hash, truncation, oversized files, replacement races, and launch failure.
- **M-03:** exact-limit and limit-plus-one tests for bytes, variants, segments, depth; cyclic manifests; streaming output overflow; live duration and cancellation.
- **M-04:** DRM fixtures for supported manifest formats, mixed protected/unprotected variants, clear refusal, and no accidental encrypted download.
- **M-08:** undefined; coverage cannot honestly be claimed.
- **I-01:** exclusion marker prevents text reads; opt-out performs no reads; URL/path-extension cases; clipboard contention; deduplication; logs contain no clipboard text.
- **I-02:** concurrent first launches, pipe squatting, wrong-user rejection, malformed/oversized requests, retry deduplication, all flags, junction escapes, and directory-swap races.
- **R-01:** expired/replayed tickets, wrong job/tab/profile, worker restart, atomic ticket consumption, changed validator, weak ETag rejection, range ignored with 200, HTML login response, and preservation of old `.part`.
- **P-01:** distinctive test password absent from settings, DB/WAL, exports, logs, and IPC; credential deletion/rotation; path-scope boundaries; redirect leakage.
- **Scheduler:** New York `2026-03-08 02:30 → 03:00`; fall-back repeated time executes once; periodic slots remain distinct; restart, sleep, clock rollback, and concurrent claims.
- **Sync:** 304 changes no file; 200 safely replaces content; metered transitions pause by default; network/auth errors preserve the previous file.