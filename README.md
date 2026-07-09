<p align="center">
  <img src="./logo.svg" alt="FerroMail Logo" width="500" />
  <br />
  <b>A high-performance, self-hosted, developer-first transactional email platform.</b>
  <br />
  <br />
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white" alt="Rust" /></a>
  <a href="https://nextjs.org"><img src="https://img.shields.io/badge/Next.js%2015-000000?style=flat-square&logo=nextdotjs&logoColor=white" alt="Next.js" /></a>
  <a href="https://www.typescriptlang.org"><img src="https://img.shields.io/badge/TypeScript-3178C6?style=flat-square&logo=typescript&logoColor=white" alt="TypeScript" /></a>
  <a href="https://react.dev"><img src="https://img.shields.io/badge/React%2019-20232A?style=flat-square&logo=react&logoColor=61DAFB" alt="React 19" /></a>
  <a href="https://postgresql.org"><img src="https://img.shields.io/badge/PostgreSQL-4169E1?style=flat-square&logo=postgresql&logoColor=white" alt="PostgreSQL" /></a>
  <a href="https://redis.io"><img src="https://img.shields.io/badge/Redis-DC382D?style=flat-square&logo=redis&logoColor=white" alt="Redis" /></a>
  <a href="https://min.io"><img src="https://img.shields.io/badge/MinIO-C72C48?style=flat-square&logo=minio&logoColor=white" alt="MinIO" /></a>
  <a href="https://docker.com"><img src="https://img.shields.io/badge/Docker-2496ED?style=flat-square&logo=docker&logoColor=white" alt="Docker" /></a>
  <a href="https://tailwindcss.com"><img src="https://img.shields.io/badge/Tailwind%20CSS-38B2AC?style=flat-square&logo=tailwindcss&logoColor=white" alt="Tailwind CSS" /></a>
</p>

---

> [!WARNING]
> **Development Notice:** FerroMail is currently a work-in-progress and is incomplete. Some features and modules may be partially functional or non-functional. The codebase will be revisited for further active development in the future.

---

Ferromail is a high-performance, self-hosted email platform designed as a modern, lightweight, and fast alternative to Plunk. It features a high-throughput backend workspace written in Rust and a premium, responsive dashboard frontend built in Next.js 15.


---

## What It's For

Ferromail is built for developers and businesses that want full control over their email infrastructure without the cost or data-sharing concerns of third-party SaaS providers. It provides:

*   **Transactional Email Delivery**: Send emails programmatically via a unified REST API or standard SMTP relay.
*   **Audience & Subscription Management**: Organize contacts, upload subscriber lists via CSV, manage opt-in/opt-out statuses, and partition contacts using dynamic metadata queries.
*   **Newsletter & Marketing Campaigns**: Build, preview, schedule, and broadcast content to targeted recipient groups.
*   **Visual Automation Workflows**: Design graph-based automation trees containing triggers, timed delays, transactional template emails, and webhook actions.
*   **Detailed Analytics & Tracking**: Monitor outbound email metrics including delivery success rates, bounce logs, open/click counts, and custom webhook dispatch.

---

## How It Works

Ferromail splits its operations into an API server, an inbound SMTP relay, and a background task worker:

```mermaid
graph TD
    Client[REST Client / SMTP Client] -->|API Call / SMTP Send| API[REST API Server: Axum]
    InboundSMTP[Inbound SMTP: mailin] -->|HTTP Relay| API
    API -->|Queue PENDING Email| DB[(PostgreSQL Database)]
    DB -->|Fetch Queue| Worker[Background Worker: Tokio]
    Worker -->|Send Outgoing MIME Mail| SMTPRelay[External Outbound SMTP Relay / AWS SES]
    Worker -->|Log Status & Event| DB
    Dashboard[Next.js Web UI] -->|Manage Contacts/Workflows/Stats| API
```

1.  **Ingestion & Queueing**:
    When an email is triggered (either via a `/v1/send` REST API post, an inbound SMTP submission, a scheduled marketing campaign, or a workflow execution node), the REST API server validates the request and queues a record in the PostgreSQL database with a status of `PENDING`.
2.  **Background Processing**:
    The background worker constantly polls the database for `PENDING` emails. It compiles templates by parsing dynamic variables (e.g. `{{ firstName ?? Guest }}`), verifies if the recipient is subscribed, checks for unsubscribe signals, and builds a standard MIME message structure.
3.  **Outbound Transmission**:
    - **Amazon SES API Mode**: If AWS environment variables are set (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION`), the worker dispatches the email using the Amazon SES v2 API. It sends the formatted raw MIME message to SES to handle attachments, custom headers, and templates natively.
    - **Outbound SMTP Relay Mode**: Otherwise, if SMTP relay variables are configured, the worker connects to the external SMTP server using `lettre` to dispatch the email.
    - **Simulation Mode**: If neither is configured, the worker operates in simulation mode, automatically updating the database records to `DELIVERED` with mock IDs.
4.  **Tracking & Events**:
    Every email delivery attempt updates the database state (`status`, `sentAt`, `deliveredAt`, `error`) and generates event logs in the `events` table (e.g. `email.delivered` or `email.failed`), updating the real-time analytics dashboard.

---

## Screenshots

<p align="center">
  <img src="./ferromail-signup.png" alt="Sign Up" width="48%" />
  <img src="./ferromail-signin.png" alt="Sign In" width="48%" />
</p>
<p align="center">
  <img src="./ferromail-dashboard.png" alt="Dashboard" width="97%" />
</p>

---

## Technology Stack

### Backend Workspace (Rust)
*   **Core Framework**: [Axum](https://github.com/tokio-rs/axum) for high-performance REST API routing.
*   **Async Runtime**: [Tokio](https://tokio.rs/) for managing concurrent loop execution in the daemon.
*   **Database Querying**: [SQLx](https://github.com/launchbadge/sqlx) for type-safe, asynchronous PostgreSQL integration.
*   **Inbound SMTP Relay**: [mailin-embedded](https://github.com/Wassasin/mailin) for parsing and handling incoming SMTP handshakes.
*   **Outbound SMTP Transport**: [lettre](https://lettre.at/) for constructing MIME messages (including attachments and custom headers) and relaying them to external mail agents.
*   **Amazon SES Integration**: [aws-sdk-sesv2](https://github.com/awslabs/aws-sdk-rust) and [aws-config](https://github.com/awslabs/aws-sdk-rust) for domain verification and direct email delivery.

### Frontend Web Client (Next.js)
*   **Core UI**: [Next.js 15](https://nextjs.org/) (App Router) and [React 19](https://react.dev/).
*   **Styling**: Vanilla CSS incorporating a glassmorphic design token system with [Tailwind CSS](https://tailwindcss.com/) for fluid layouts.
*   **Typography**: Google Fonts (`Outfit` for headers, `Inter` for general copy, and `JetBrains Mono` for code snippets).
*   **Icons**: [Lucide React](https://lucide.dev/).

---

## Environment Variables

### Backend (`backend/.env`)

| Variable | Description | Default / Example |
| :--- | :--- | :--- |
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://postgres:postgres@localhost:55432/postgres` |
| `JWT_SECRET` | Signing key for user session tokens | `ferromail_jwt_signing_secret_key` |
| `API_URI` | Base public URI for the REST API | `http://localhost:3000` |
| `SMTP_DOMAIN` | Domain host used by the inbound SMTP server | `localhost` |
| **Amazon SES Settings** | | |
| `AWS_ACCESS_KEY_ID` | Access key for AWS IAM user (Enables live Amazon SES integration) | `AKIAIOSFODNN7EXAMPLE` |
| `AWS_SECRET_ACCESS_KEY`| Secret key for AWS IAM user | `wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY` |
| `AWS_REGION` | AWS region hosting your SES endpoint | `us-east-1` |
| **Outbound SMTP Settings** | | |
| `SMTP_RELAY_HOST` | Host of the external SMTP relay (e.g. SendGrid, Mailgun) | `smtp.mailgun.org` (or `SMTP_HOST`) |
| `SMTP_RELAY_PORT` | Port of the external SMTP relay | `587` or `465` (or `SMTP_PORT`) |
| `SMTP_RELAY_USERNAME` | Authentication username for the SMTP relay | `postmaster@domain.com` (or `SMTP_USER`) |
| `SMTP_RELAY_PASSWORD` | Authentication password/key for the SMTP relay | `your_smtp_password` (or `SMTP_PASS`) |
| `SMTP_RELAY_SECURE` | TLS settings (`true`, `false`, or `opportunistic`) | `true` (or `SMTP_SECURE`) |

### Frontend (`frontend/.env`)

| Variable | Description | Default / Example |
| :--- | :--- | :--- |
| `NEXT_PUBLIC_API_URL` | API endpoint for frontend server/client fetches | `http://localhost:3000` |
| `PORT` | Listening port for the Next.js server | `4000` |

---

## Setup Instructions

### Option A: Run the Entire Stack in Docker (Recommended)

To boot up the entire ecosystem (database, cache, file storage, API router, SMTP inbound server, worker, and Next.js client) run the following command in the root folder:

```powershell
docker compose up --build -d
```

Once running:
*   **Frontend UI Dashboard**: `http://localhost:4000`
*   **REST API Engine**: `http://localhost:3000`
*   **MinIO Console (S3 storage)**: `http://localhost:9001` (Username: `plunk`, Password: `plunkminiopass`)

To configure real outbound mail, define the `SMTP_RELAY_*` environment variables in your terminal shell before starting the docker composition.

To shut down:
```powershell
docker compose down
```

---

### Option B: Local Development Mode (Hybrid)

To run components individually for developer changes:

#### 1. Launch Infrastructure
Spin up PostgreSQL, Redis, and MinIO in Docker:
```powershell
docker compose -f docker/docker-compose.dev.yml up -d
```

#### 2. Configure Environment
Create a `.env` file in the `backend` folder following the variables schema above.

#### 3. Run Components Locally (in separate terminals)

**REST API Server:**
```powershell
cd backend
cargo run --package api
```

**Background Worker Daemon:**
```powershell
cd backend
cargo run --package worker
```

**Inbound SMTP Server:**
```powershell
cd backend
cargo run --package smtp
```

**Frontend Dashboard:**
```powershell
cd frontend
npm install
npm run dev
```

---

## Verification & Integration Testing

To run the automated end-to-end integration test suite, run:

```powershell
node e2e_test.js
```

This script automatically verifies:
1.  Successful Rust workspace compilation.
2.  REST API signup and membership provisioning.
3.  CSV contact insertion.
4.  Template rendering and placeholder compilation.
5.  Tokio worker background queue polling.
6.  DB state updates (updating records to `DELIVERED`).
7.  Outbound audit event log registration.

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
