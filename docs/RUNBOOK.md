# GEPA v2 Operational Runbook

This runbook provides step-by-step procedures for administering, operating, deploying, and maintaining the GEPA (General English Proficiency Assessment) v2 platform.

---

## 1. System Architecture & Health Check

### 1.1 Architecture Overview
- **Pure Engine (`shared/engine`):** Synchronous, deterministic Rust scoring crate. Evaluates routing, trait models, confidence, and CEFR band outcomes.
- **Backend API Server (`server/`):** Tokio + Axum REST API, handles session lifecycle, signed media delivery, and AI rater pipelines.
- **Client Application (`client/`):** Astro.js accessible static site with interactive WCAG 2.2 AA compliant test delivery islands.
- **AI Scoring (`server/src/ai/`):** Multimodal Gemini rating engine for speaking and writing tasks with prompt versioning and rubric checks.

### 1.2 Health Check Endpoint
To verify server liveness and service health:
```bash
curl -f http://localhost:3000/healthz
```
Expected response:
```json
{
  "status": "ok",
  "version": "2.0.0-beta",
  "timestamp": "2026-09-08T22:45:00Z"
}
```

---

## 2. API Key & Secret Rotation

### 2.1 Google Gemini API Key (`GEMINI_API_KEY`)
1. Generate a new API key in Google AI Studio or Google Cloud Console.
2. In production (Google Cloud Run):
   ```bash
   gcloud secrets versions add gepa-gemini-api-key --data-file=- <<< "NEW_KEY"
   gcloud run services update gepa-server \
     --update-secrets=GEMINI_API_KEY=gepa-gemini-api-key:latest \
     --region=europe-west1
   ```
3. In local staging:
   Update `.env` or set environment variable:
   ```powershell
   $env:GEMINI_API_KEY = "NEW_KEY"
   ```
4. Verification: Run a test writing or speaking submission to confirm AI rating pipeline functions without quota or authentication errors.

### 2.2 Media Signing Secret / Token (`MEDIA_SIGNING_SECRET`)
Signed audio URLs expire after 15 minutes. To rotate the HMAC key:
1. Update `MEDIA_SIGNING_SECRET` environment variable in secret manager.
2. Grace period: The server accepts tokens signed by both previous and current secret for a 15-minute overlap window.

---

## 3. Seed Bank Validation & Loading

### 3.1 Pre-Deployment Invariant Check
Before loading any item bank or stimuli updates into the system, validate all checksums, schemas, and counts:
```bash
npm run seed:validate
```
Must pass with:
- 52 Language Systems items
- 42 Reading items across 21 stimuli
- 42 Listening items across 21 stimuli
- 48 Speaking tasks (6 SR, 6 RT, 12 INT1, 12 INT2)
- 24 Writing tasks (6 L2W, 6 ER1, 6 ER2, 6 ER3)
- 136 Answer keys matching authoring checksums

### 3.2 Form & Bank Balance Check
Verify enemy group conflicts and option letter balance:
```bash
npm run forms:check
```
Invariants verified:
- No authoring letter exceeds 40% across any module.
- All 4 communicative CEFR domains (public, educational, personal, occupational) are represented.
- 0 enemy group collisions across 22 defined conflict sets.

### 3.3 Loading Seed Collections (Two-Stage Isolation)
Run the dual-stage loader:
```bash
npm run seed:load
```
Boundary guarantees:
- **Candidate-Safe Collections:** `items`, `reading_stimuli`, `listening_stimuli` (with `text` transcript strictly stripped).
- **Restricted Collections (Server Only):** `restricted_keys`, `listening_admin_stimuli`. Blocked from candidate client access — the seed bank stays server-side in memory (`server/src/repos.rs::SeedBank`), never exposed via any client-reachable table or endpoint (DECISIONS.md D-021).

To hot-reload the seed bank in the running server without downtime:
```bash
curl -X POST http://localhost:3000/api/admin/seed/load \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

---

## 4. Rescoring & Human Scoring Operations

### 4.1 Audit Trail Invariant
- Ratings are **never** overwritten or deleted.
- Rescoring an existing session marks the prior rating with `superseded_by: Some(new_rating_id)`.
- Candidate result computation filters strictly active, usable ratings (`superseded_by.is_none() && usable == true`).

### 4.2 Automated Rescoring (Prompt or Rubric Upgrade)
When a new rubric version (e.g. `speaking_rater@v2.1`) is deployed:
```bash
curl -X POST http://localhost:3000/api/review/sessions/{SESSION_ID}/rescore \
  -H "Authorization: Bearer ADMIN_OR_REVIEWER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "module": "Speaking",
    "new_rubric_version": "v2.1",
    "reason": "Annual rubric recalibration and prompt refinement"
  }'
```
Response:
```json
{
  "session_id": "01K...",
  "old_rating_id": "RAT_OLD...",
  "new_rating_id": "RAT_NEW...",
  "superseded": true,
  "status": "rescored"
}
```

### 4.3 Human Expert Overwrite / Adjudication
When a flagged session requires human examiner scoring:
```bash
curl -X POST http://localhost:3000/api/review/sessions/{SESSION_ID}/human-score \
  -H "Authorization: Bearer REVIEWER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "module": "Writing",
    "traits": [
      { "trait": "Task Fulfillment", "score": 4.0 },
      { "trait": "Coherence & Cohesion", "score": 3.5 },
      { "trait": "Lexical Resource", "score": 4.0 },
      { "trait": "Grammatical Range & Accuracy", "score": 3.5 }
    ],
    "notes": "Reviewed AI-suspect flag; verified authentic human prose with idiomatic phrasing.",
    "adjudicator_id": "EXAMINER_042"
  }'
```

---

## 5. Audio Production Pipeline & QC

### 5.1 Asset Production & Loudness Normalization
To generate or verify all 52 synthesized audio assets (21 listening, 24 speaking prompts, 6 listen-to-write sentences, 1 calibration tone):
```bash
npm run audio:produce
```
QC Standards Enforced:
- Integrated Loudness: `-16.0 LUFS` (±1.0 LUFS)
- True Peak Ceiling: `≤ -1.5 dBTP`
- Silence Padding: Exactly `0.5s` lead-in, `1.0s` trailing silence
- Target Delivery Speed (WPM):
  - Pre-A1: 105 WPM
  - A1: 118 WPM
  - A2: 138 WPM
  - B1: 148 WPM
  - B2: 153 WPM
  - C1: 158 WPM
  - C2: 160 WPM
- Reports generated: `assets/qc_report.json` and `assets/qc_report.md`.

---

## 6. Exposure & Response-Time Telemetry Reports

### 6.1 Generating Operational Exposure Reports
To audit item exposure rates and mean latencies across all modules:
```bash
npm run report:exposure
```
Output: `reports/gepa_exposure_report.csv`

Privacy & Audit Guarantee:
- Zero PII (no candidate names, IP addresses, emails, or session IDs).
- Flags any item nearing the operational exposure threshold.

---

## 7. Data Retention & GDPR Lifecycle

### 7.1 Automated Retention Job
Execute the UK GDPR minimization lifecycle:
```bash
npm run retention:run
```
Policy Windows:
- **Audio Recordings:** Hard-deleted after **90 days** (unless explicit opt-in for psychometric research provided).
- **Writing Drafts & Responses:** Anonymised and retained for **24 months** for validation studies.
- **Inactive/Abandoned Sessions:** Pruned after **30 days**.
- **Aggregated Exposure Statistics:** Retained permanently (zero PII).

### 7.2 Right to be Forgotten ("Delete My Data")
When a candidate requests deletion via the results screen button or privacy contact:
1. Endpoint: `POST /api/candidate/delete-my-data`
2. Hard-deletes all associated audio blobs, text submissions, and session documents.
3. Leaves only an anonymized tombstone counter in audit telemetry.

---

## 8. Rollback Procedures

### 8.1 Rolling Back Cloud Run Deployment
If an issue occurs after a release:
```bash
gcloud run services update-traffic gepa-server \
  --to-revisions=gepa-server-PREVIOUS_TAG=100 \
  --region=europe-west1
```

### 8.2 Rolling Back Item Seed Bank
1. Revert `seed/` directory commit via Git:
   ```bash
   git checkout <PREVIOUS_COMMIT> -- seed/
   ```
2. Re-validate and reload:
   ```bash
   npm run seed:validate
   npm run seed:load
   ```
3. Issue reload signal to server:
   ```bash
   curl -X POST http://localhost:3000/api/admin/seed/load -H "Authorization: Bearer ADMIN_TOKEN"
   ```

### 8.3 Rolling Back AI Prompt or Rubric Versions
The prompt registry in `server/src/ai/prompts.rs` is append-only with explicit version tags (`speaking_rater@v2.0`, `speaking_rater@v1.9`). To roll back:
1. Adjust the default version constant in `server/src/ai/mod.rs` to the prior tag.
2. Re-deploy backend service.
3. Rescore any sessions evaluated during the faulty window using the rescoring API.

---

## 9. Access Control & Deployment

### 9.1 Ownership/role enforcement (DECISIONS.md D-021)
There is no separate rules layer — Postgres has no equivalent to Firestore Security Rules, so the same checks that
were always the real enforcement point for anything routed through the API are the *only* enforcement point:
- `server/src/api.rs::require_session_owner` — every `/api/sessions/:id/*` route: the session must exist and the
  caller must own it (candidate token `sub` matches `sessions.candidate_uid`) or be reviewer/admin staff.
- `server/src/auth.rs::AuthUser` / `StaffAuth` / `AdminAuth` — Axum extractors gating every route by role
  (`/api/review/*` needs reviewer or admin, `/api/admin/*` needs admin).
- The seed bank (`restricted_keys`, listening scripts) never leaves server memory — no table or endpoint exposes it
  to any client role, staff included.

### 9.2 Postgres schema
Migrations are one file, `server/migrations/001_init.sql`, embedded into the server binary and run idempotently at
every startup — no separate deploy step. To apply manually against a specific database:
```bash
psql "$DATABASE_URL" -f server/migrations/001_init.sql
```

### 9.3 Provisioning a reviewer/admin account
```bash
npm run staff:create -- reviewer@example.com "a-strong-password" reviewer
npm run staff:create -- admin@example.com "a-strong-password" admin
```
Re-running with the same email resets that account's password/role.

### 9.4 Candidate-view sanitization & leak gates (DECISIONS.md D-022)
The productive task endpoints strip server-only fields (`audio_script`, `interlocutor_line`) at the service
boundary before serialization — the client schemas no longer even declare them. If you add a new
candidate-facing field to `SpeakingTask`/`WritingTask`, ask whether it's rating material first; if it is,
null it out in `get_speaking_tasks`/`get_writing_tasks` (server/src/services.rs) the same way. Three CI
gates enforce this continuously:
- `security.spec.ts` scans every `/api/*` response across a full journey (including `/speaking/start` and
  `/writing/start`) for the field *names* `audio_script`/`interlocutor_line`/`key_option_id`/
  `authoring_letter` and for verbatim seed script text — a leak fails CI, not just review.
- `scripts/scan_bundle_secrets.py` scans the built `client/dist` for the same material plus
  secret-shaped strings.
- `scripts/check_bundle_size.py` holds the initial gzipped payload under the 400 kB spec budget
  (currently 84 kB).

### 9.5 Logging redaction (AGENTS.md §3)
Two layers, both in CI:
- Static: `server/src/logging.rs::log_statement_source_scan` fails `cargo test` if any `tracing::*!` macro
  in `server/src` interpolates a redaction-list identifier (audio URLs, transcripts, option ids, keys,
  emails, credentials, DSNs). Prose inside the message literal is fine — only interpolated identifiers trip it.
- Runtime: the CI container job boots the image with canary secrets and a wrong DB password (exercising
  the pool/migration failure paths) and greps the container's full log output for them. The Gemini API key
  rides in the `x-goog-api-key` header, never the URL, so reqwest error Display strings cannot embed it.

### 9.6 Go-live checklist — shared-platform VPS (DECISIONS.md D-023)
GEPA deploys to the shared production box running the **platform stack** (`/opt/platform`): one shared
Postgres instance for every project on the box, rules in `/opt/platform/PLATFORM-RULES.md` (VPS-only doc;
same pattern UBAG follows). Consequently `docker-compose.prod.yml` contains **no database service**: the
server joins the external `platform` docker network and consumes `platform-postgres`. Ingress is the shared
Nginx Proxy Manager over the external `nginx-proxy-manager_default` network with **no host ports published**
— the box has no active firewall, so any 0.0.0.0-bound port would be directly internet-reachable. The VPS
address itself is deliberately not written here; this repo is public.

One-time platform provisioning (on the VPS, platform admin):
```bash
# Creates the gepa database + least-privilege user on the shared instance and writes
# /opt/platform/projects/gepa.env with DATABASE_URL (never committed to this repo).
/opt/platform/bin/provision-project.sh gepa

# Append the app secrets the platform does not manage:
cat >> /opt/platform/projects/gepa.env
JWT_SECRET=$(openssl rand -base64 48)
GEMINI_API_KEY=<key>   # optional: enables AI scoring + the TTS audio upgrade (Q-002, Q-007)
```

One-time repo secrets (GitHub → Settings → Secrets and variables → Actions):
`VPS_HOST` (the shared box's address), `VPS_SSH_KEY` (deploy private key), optional
`VPS_USERNAME`/`VPS_PORT` (defaults `root`/`22`).

One-time ingress (NPM admin UI, same pattern as other projects on the box): Proxy Host —
domain `gepa.<your-domain>` (DNS via Cloudflare), Forward Hostname `gepa-server-prod`,
Forward Port `8080`, new Let's Encrypt certificate, force SSL, HTTP/2.

Deploy: `git tag v2.0.0-beta.5 && git push origin v2.0.0-beta.5` → `deploy-vps.yml` builds the
image on GitHub Actions runners, publishes `ghcr.io/jerryboganda/gepa/gepa-server` (semver +
`sha-` + `latest` tags), then the VPS pulls it (zero compute), starts it with the platform env
file, and the workflow smokes `/healthz` via `docker compose exec` (no host ports exist). The
GHCR package must stay **private**: the image carries `RESTRICTED_*` seed assets (answer keys);
the workflow authenticates its pull with a short-lived workflow token — never make it public.

---

## 10. Zero-Compute Production VPS Policy & Operations

### 10.1 Core Policy
The production VPS is dedicated solely to serving live candidate sessions with minimal CPU, memory, and disk pressure. It must **never** be used as a general-purpose compute machine or burdened with:
- `cargo build`, `cargo test`, `cargo clippy`, or Rust compilation.
- `npm run build`, Astro SSG generation, or client bundling.
- Docker image building or layer composition.
- Heavy batch data processing or seed generation.

### 10.2 Compute Offloading to GitHub Actions
All resource-intensive jobs are offloaded to GitHub Actions runners:
1. **Continuous Integration (`.github/workflows/ci.yml`)**:
   - Compiles Rust and Astro assets.
   - Runs full test suites, typechecks, and claims copy-linting.
   - Executes item bank validations (`seed:validate`, `forms:check`, `audio:produce`, `report:exposure`).
   - Builds multi-stage Docker images with GitHub Actions caching (`type=gha`).
2. **Cloud Run Deployment (`.github/workflows/deploy-cloud-run.yml`)**:
   - Builds production container in GitHub Actions and deploys the pre-built image to Google Cloud Run.
3. **VPS Zero-Compute Deployment (`.github/workflows/deploy-vps.yml`)**:
   - Builds and publishes the pre-built runtime container to GitHub Container Registry (`ghcr.io`).
   - Triggers the VPS via SSH to copy the exact tagged compose file and run
     `docker compose -f docker-compose.prod.yml --env-file /opt/platform/projects/gepa.env pull && up -d`.
     No per-app database: Postgres is the shared platform instance over the external `platform`
     network (§9.6, D-023). The smoke check runs via `docker compose exec` because no host ports
     are published (shared Nginx Proxy Manager handles ingress).
4. **Scheduled Maintenance (`.github/workflows/scheduled-maintenance.yml`)**:
   - Runs daily at 02:00 UTC on GitHub Actions runners for GDPR retention purges and telemetry exports.

### 10.3 Manual VPS Deployment Steps (Zero-Compute, Shared Platform Profile)
If deploying or updating manually on a production VPS (one-time platform provisioning per §9.6):
```bash
cd /opt/docker/gepa   # compose file copied here by the deploy workflow (or scp it manually)

# 1. Pull pre-compiled container image directly (Zero CPU/memory compilation)
docker compose -f docker-compose.prod.yml --env-file /opt/platform/projects/gepa.env pull

# 2. Start or restart the container with resource limits (1 CPU, 512MB RAM cap), no host ports
docker compose -f docker-compose.prod.yml --env-file /opt/platform/projects/gepa.env up -d --remove-orphans

# 3. Prune dangling images to ensure zero disk creep
docker image prune -f

# 4. Verify live application health (no published ports — exec inside the compose network)
docker compose -f docker-compose.prod.yml --env-file /opt/platform/projects/gepa.env \
  exec -T gepa-server curl -f http://localhost:8080/healthz
```

