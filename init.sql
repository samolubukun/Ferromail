--
-- PostgreSQL database dump
--


-- Dumped from database version 16.14
-- Dumped by pg_dump version 16.14

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: AuthMethod; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public."AuthMethod" AS ENUM (
    'PASSWORD',
    'GOOGLE_OAUTH',
    'GITHUB_OAUTH'
);


ALTER TYPE public."AuthMethod" OWNER TO postgres;

--
-- Name: CampaignAudienceType; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public."CampaignAudienceType" AS ENUM (
    'ALL',
    'FILTERED',
    'SEGMENT'
);


ALTER TYPE public."CampaignAudienceType" OWNER TO postgres;

--
-- Name: CampaignStatus; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public."CampaignStatus" AS ENUM (
    'DRAFT',
    'SCHEDULED',
    'SENDING',
    'SENT',
    'CANCELLED'
);


ALTER TYPE public."CampaignStatus" OWNER TO postgres;

--
-- Name: EmailSourceType; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public."EmailSourceType" AS ENUM (
    'TRANSACTIONAL',
    'CAMPAIGN',
    'WORKFLOW',
    'INBOUND'
);


ALTER TYPE public."EmailSourceType" OWNER TO postgres;

--
-- Name: EmailStatus; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public."EmailStatus" AS ENUM (
    'PENDING',
    'SENDING',
    'SENT',
    'DELIVERED',
    'OPENED',
    'CLICKED',
    'BOUNCED',
    'COMPLAINED',
    'FAILED',
    'RECEIVED'
);


ALTER TYPE public."EmailStatus" OWNER TO postgres;

--
-- Name: ProjectDisabledReason; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public."ProjectDisabledReason" AS ENUM (
    'PAYMENT_FAILED',
    'EMAIL_REPUTATION',
    'PHISHING_DETECTED',
    'MANUAL'
);


ALTER TYPE public."ProjectDisabledReason" OWNER TO postgres;

--
-- Name: Role; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public."Role" AS ENUM (
    'OWNER',
    'ADMIN',
    'MEMBER'
);


ALTER TYPE public."Role" OWNER TO postgres;

--
-- Name: SegmentType; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public."SegmentType" AS ENUM (
    'DYNAMIC',
    'STATIC'
);


ALTER TYPE public."SegmentType" OWNER TO postgres;

--
-- Name: StepExecutionStatus; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public."StepExecutionStatus" AS ENUM (
    'PENDING',
    'SCHEDULED',
    'WAITING',
    'RUNNING',
    'COMPLETED',
    'SKIPPED',
    'FAILED'
);


ALTER TYPE public."StepExecutionStatus" OWNER TO postgres;

--
-- Name: TemplateType; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public."TemplateType" AS ENUM (
    'TRANSACTIONAL',
    'MARKETING',
    'HEADLESS'
);


ALTER TYPE public."TemplateType" OWNER TO postgres;

--
-- Name: TrackingMode; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public."TrackingMode" AS ENUM (
    'ENABLED',
    'DISABLED',
    'MARKETING_ONLY'
);


ALTER TYPE public."TrackingMode" OWNER TO postgres;

--
-- Name: WorkflowExecutionStatus; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public."WorkflowExecutionStatus" AS ENUM (
    'RUNNING',
    'WAITING',
    'COMPLETED',
    'EXITED',
    'FAILED',
    'CANCELLED'
);


ALTER TYPE public."WorkflowExecutionStatus" OWNER TO postgres;

--
-- Name: WorkflowStepType; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public."WorkflowStepType" AS ENUM (
    'TRIGGER',
    'SEND_EMAIL',
    'DELAY',
    'WAIT_FOR_EVENT',
    'CONDITION',
    'EXIT',
    'WEBHOOK',
    'UPDATE_CONTACT'
);


ALTER TYPE public."WorkflowStepType" OWNER TO postgres;

--
-- Name: WorkflowTriggerType; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public."WorkflowTriggerType" AS ENUM (
    'EVENT',
    'MANUAL',
    'SCHEDULE'
);


ALTER TYPE public."WorkflowTriggerType" OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: _prisma_migrations; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public._prisma_migrations (
    id character varying(36) NOT NULL,
    checksum character varying(64) NOT NULL,
    finished_at timestamp with time zone,
    migration_name character varying(255) NOT NULL,
    logs text,
    rolled_back_at timestamp with time zone,
    started_at timestamp with time zone DEFAULT now() NOT NULL,
    applied_steps_count integer DEFAULT 0 NOT NULL
);


ALTER TABLE public._prisma_migrations OWNER TO postgres;

--
-- Name: api_requests; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.api_requests (
    id text NOT NULL,
    method text NOT NULL,
    path text NOT NULL,
    "statusCode" integer NOT NULL,
    duration integer NOT NULL,
    "projectId" text,
    "userId" text,
    "authType" text,
    ip text,
    "userAgent" text,
    "errorCode" text,
    "errorMessage" text,
    "requestSize" integer,
    "responseSize" integer,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.api_requests OWNER TO postgres;

--
-- Name: campaigns; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.campaigns (
    id text NOT NULL,
    name text NOT NULL,
    description text,
    status public."CampaignStatus" DEFAULT 'DRAFT'::public."CampaignStatus" NOT NULL,
    subject text NOT NULL,
    body text NOT NULL,
    "from" text NOT NULL,
    "fromName" text,
    "replyTo" text,
    "audienceType" public."CampaignAudienceType" DEFAULT 'ALL'::public."CampaignAudienceType" NOT NULL,
    "audienceCondition" jsonb,
    "segmentId" text,
    "scheduledFor" timestamp(3) without time zone,
    "totalRecipients" integer DEFAULT 0 NOT NULL,
    "sentCount" integer DEFAULT 0 NOT NULL,
    "deliveredCount" integer DEFAULT 0 NOT NULL,
    "openedCount" integer DEFAULT 0 NOT NULL,
    "clickedCount" integer DEFAULT 0 NOT NULL,
    "bouncedCount" integer DEFAULT 0 NOT NULL,
    "projectId" text NOT NULL,
    "sentAt" timestamp(3) without time zone,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL,
    type public."TemplateType" DEFAULT 'MARKETING'::public."TemplateType" NOT NULL
);


ALTER TABLE public.campaigns OWNER TO postgres;

--
-- Name: contacts; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.contacts (
    id text NOT NULL,
    email text NOT NULL,
    data jsonb,
    subscribed boolean DEFAULT true NOT NULL,
    "projectId" text NOT NULL,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL
);


ALTER TABLE public.contacts OWNER TO postgres;

--
-- Name: domains; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.domains (
    id text NOT NULL,
    domain text NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    "dkimTokens" jsonb,
    "projectId" text NOT NULL,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL
);


ALTER TABLE public.domains OWNER TO postgres;

--
-- Name: emails; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.emails (
    id text NOT NULL,
    "contactId" text NOT NULL,
    "toName" text,
    subject text NOT NULL,
    body text NOT NULL,
    "from" text NOT NULL,
    "fromName" text,
    "replyTo" text,
    headers jsonb,
    attachments jsonb,
    "messageId" text,
    "sourceType" public."EmailSourceType" NOT NULL,
    "templateId" text,
    "campaignId" text,
    "workflowExecutionId" text,
    "workflowStepExecutionId" text,
    status public."EmailStatus" DEFAULT 'PENDING'::public."EmailStatus" NOT NULL,
    "sentAt" timestamp(3) without time zone,
    "deliveredAt" timestamp(3) without time zone,
    "openedAt" timestamp(3) without time zone,
    "clickedAt" timestamp(3) without time zone,
    "bouncedAt" timestamp(3) without time zone,
    "complainedAt" timestamp(3) without time zone,
    opens integer DEFAULT 0 NOT NULL,
    clicks integer DEFAULT 0 NOT NULL,
    error text,
    "projectId" text NOT NULL,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL
);


ALTER TABLE public.emails OWNER TO postgres;

--
-- Name: events; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.events (
    id text NOT NULL,
    name text NOT NULL,
    data jsonb,
    "projectId" text NOT NULL,
    "contactId" text,
    "emailId" text,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.events OWNER TO postgres;

--
-- Name: memberships; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.memberships (
    "userId" text NOT NULL,
    "projectId" text NOT NULL,
    role public."Role" DEFAULT 'MEMBER'::public."Role" NOT NULL,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL
);


ALTER TABLE public.memberships OWNER TO postgres;

--
-- Name: projects; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.projects (
    id text NOT NULL,
    name text NOT NULL,
    public text NOT NULL,
    secret text NOT NULL,
    disabled boolean DEFAULT false NOT NULL,
    customer text,
    subscription text,
    "billingLimitWorkflows" integer,
    "billingLimitCampaigns" integer,
    "billingLimitTransactional" integer,
    tracking public."TrackingMode" DEFAULT 'ENABLED'::public."TrackingMode" NOT NULL,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL,
    language text DEFAULT 'en'::text NOT NULL,
    "billingLimitInbound" integer,
    "disabledReason" public."ProjectDisabledReason"
);


ALTER TABLE public.projects OWNER TO postgres;

--
-- Name: segment_memberships; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.segment_memberships (
    "contactId" text NOT NULL,
    "segmentId" text NOT NULL,
    "enteredAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "exitedAt" timestamp(3) without time zone,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL
);


ALTER TABLE public.segment_memberships OWNER TO postgres;

--
-- Name: segments; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.segments (
    id text NOT NULL,
    name text NOT NULL,
    description text,
    condition jsonb,
    "trackMembership" boolean DEFAULT false NOT NULL,
    "memberCount" integer DEFAULT 0 NOT NULL,
    "projectId" text NOT NULL,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL,
    type public."SegmentType" DEFAULT 'DYNAMIC'::public."SegmentType" NOT NULL
);


ALTER TABLE public.segments OWNER TO postgres;

--
-- Name: templates; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.templates (
    id text NOT NULL,
    name text NOT NULL,
    description text,
    subject text NOT NULL,
    body text NOT NULL,
    "from" text NOT NULL,
    "fromName" text,
    "replyTo" text,
    type public."TemplateType" DEFAULT 'MARKETING'::public."TemplateType" NOT NULL,
    "projectId" text NOT NULL,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL
);


ALTER TABLE public.templates OWNER TO postgres;

--
-- Name: users; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.users (
    id text NOT NULL,
    email text NOT NULL,
    password text,
    type public."AuthMethod" NOT NULL,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL,
    "emailVerified" boolean DEFAULT false NOT NULL
);


ALTER TABLE public.users OWNER TO postgres;

--
-- Name: workflow_executions; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.workflow_executions (
    id text NOT NULL,
    "workflowId" text NOT NULL,
    "contactId" text NOT NULL,
    status public."WorkflowExecutionStatus" DEFAULT 'RUNNING'::public."WorkflowExecutionStatus" NOT NULL,
    "currentStepId" text,
    "exitReason" text,
    context jsonb,
    "startedAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "completedAt" timestamp(3) without time zone,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL
);


ALTER TABLE public.workflow_executions OWNER TO postgres;

--
-- Name: workflow_step_executions; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.workflow_step_executions (
    id text NOT NULL,
    "executionId" text NOT NULL,
    "stepId" text NOT NULL,
    status public."StepExecutionStatus" DEFAULT 'PENDING'::public."StepExecutionStatus" NOT NULL,
    "scheduledFor" timestamp(3) without time zone,
    "executeAfter" timestamp(3) without time zone,
    output jsonb,
    error text,
    "startedAt" timestamp(3) without time zone,
    "completedAt" timestamp(3) without time zone,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL
);


ALTER TABLE public.workflow_step_executions OWNER TO postgres;

--
-- Name: workflow_steps; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.workflow_steps (
    id text NOT NULL,
    type public."WorkflowStepType" NOT NULL,
    name text NOT NULL,
    "position" jsonb NOT NULL,
    config jsonb NOT NULL,
    "workflowId" text NOT NULL,
    "templateId" text,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL
);


ALTER TABLE public.workflow_steps OWNER TO postgres;

--
-- Name: workflow_transitions; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.workflow_transitions (
    id text NOT NULL,
    "fromStepId" text NOT NULL,
    "toStepId" text NOT NULL,
    condition jsonb,
    priority integer DEFAULT 0 NOT NULL,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL
);


ALTER TABLE public.workflow_transitions OWNER TO postgres;

--
-- Name: workflows; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.workflows (
    id text NOT NULL,
    name text NOT NULL,
    description text,
    enabled boolean DEFAULT false NOT NULL,
    "triggerType" public."WorkflowTriggerType" NOT NULL,
    "triggerConfig" jsonb,
    "allowReentry" boolean DEFAULT false NOT NULL,
    "projectId" text NOT NULL,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" timestamp(3) without time zone NOT NULL
);


ALTER TABLE public.workflows OWNER TO postgres;

--
-- Name: _prisma_migrations _prisma_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public._prisma_migrations
    ADD CONSTRAINT _prisma_migrations_pkey PRIMARY KEY (id);


--
-- Name: api_requests api_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.api_requests
    ADD CONSTRAINT api_requests_pkey PRIMARY KEY (id);


--
-- Name: campaigns campaigns_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.campaigns
    ADD CONSTRAINT campaigns_pkey PRIMARY KEY (id);


--
-- Name: contacts contacts_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.contacts
    ADD CONSTRAINT contacts_pkey PRIMARY KEY (id);


--
-- Name: domains domains_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.domains
    ADD CONSTRAINT domains_pkey PRIMARY KEY (id);


--
-- Name: emails emails_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.emails
    ADD CONSTRAINT emails_pkey PRIMARY KEY (id);


--
-- Name: events events_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.events
    ADD CONSTRAINT events_pkey PRIMARY KEY (id);


--
-- Name: memberships memberships_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.memberships
    ADD CONSTRAINT memberships_pkey PRIMARY KEY ("userId", "projectId");


--
-- Name: projects projects_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT projects_pkey PRIMARY KEY (id);


--
-- Name: segment_memberships segment_memberships_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.segment_memberships
    ADD CONSTRAINT segment_memberships_pkey PRIMARY KEY ("contactId", "segmentId");


--
-- Name: segments segments_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.segments
    ADD CONSTRAINT segments_pkey PRIMARY KEY (id);


--
-- Name: templates templates_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.templates
    ADD CONSTRAINT templates_pkey PRIMARY KEY (id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: workflow_executions workflow_executions_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflow_executions
    ADD CONSTRAINT workflow_executions_pkey PRIMARY KEY (id);


--
-- Name: workflow_step_executions workflow_step_executions_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflow_step_executions
    ADD CONSTRAINT workflow_step_executions_pkey PRIMARY KEY (id);


--
-- Name: workflow_steps workflow_steps_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflow_steps
    ADD CONSTRAINT workflow_steps_pkey PRIMARY KEY (id);


--
-- Name: workflow_transitions workflow_transitions_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflow_transitions
    ADD CONSTRAINT workflow_transitions_pkey PRIMARY KEY (id);


--
-- Name: workflows workflows_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflows
    ADD CONSTRAINT workflows_pkey PRIMARY KEY (id);


--
-- Name: api_requests_createdAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "api_requests_createdAt_idx" ON public.api_requests USING btree ("createdAt" DESC);


--
-- Name: api_requests_path_createdAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "api_requests_path_createdAt_idx" ON public.api_requests USING btree (path, "createdAt" DESC);


--
-- Name: api_requests_projectId_createdAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "api_requests_projectId_createdAt_idx" ON public.api_requests USING btree ("projectId", "createdAt" DESC);


--
-- Name: api_requests_projectId_statusCode_createdAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "api_requests_projectId_statusCode_createdAt_idx" ON public.api_requests USING btree ("projectId", "statusCode", "createdAt" DESC);


--
-- Name: api_requests_statusCode_createdAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "api_requests_statusCode_createdAt_idx" ON public.api_requests USING btree ("statusCode", "createdAt" DESC);


--
-- Name: api_requests_userId_createdAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "api_requests_userId_createdAt_idx" ON public.api_requests USING btree ("userId", "createdAt" DESC);


--
-- Name: campaigns_audienceCondition_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "campaigns_audienceCondition_idx" ON public.campaigns USING gin ("audienceCondition");


--
-- Name: campaigns_projectId_status_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "campaigns_projectId_status_idx" ON public.campaigns USING btree ("projectId", status);


--
-- Name: campaigns_scheduledFor_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "campaigns_scheduledFor_idx" ON public.campaigns USING btree ("scheduledFor");


--
-- Name: campaigns_segmentId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "campaigns_segmentId_idx" ON public.campaigns USING btree ("segmentId");


--
-- Name: contacts_data_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX contacts_data_idx ON public.contacts USING gin (data);


--
-- Name: contacts_projectId_email_key; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX "contacts_projectId_email_key" ON public.contacts USING btree ("projectId", email);


--
-- Name: contacts_projectId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "contacts_projectId_idx" ON public.contacts USING btree ("projectId");


--
-- Name: contacts_projectId_subscribed_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "contacts_projectId_subscribed_idx" ON public.contacts USING btree ("projectId", subscribed);


--
-- Name: domains_projectId_domain_key; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX "domains_projectId_domain_key" ON public.domains USING btree ("projectId", domain);


--
-- Name: domains_projectId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "domains_projectId_idx" ON public.domains USING btree ("projectId");


--
-- Name: domains_projectId_verified_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "domains_projectId_verified_idx" ON public.domains USING btree ("projectId", verified);


--
-- Name: emails_campaignId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "emails_campaignId_idx" ON public.emails USING btree ("campaignId");


--
-- Name: emails_campaignId_status_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "emails_campaignId_status_idx" ON public.emails USING btree ("campaignId", status);


--
-- Name: emails_contactId_bouncedAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "emails_contactId_bouncedAt_idx" ON public.emails USING btree ("contactId", "bouncedAt");


--
-- Name: emails_contactId_clickedAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "emails_contactId_clickedAt_idx" ON public.emails USING btree ("contactId", "clickedAt");


--
-- Name: emails_contactId_complainedAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "emails_contactId_complainedAt_idx" ON public.emails USING btree ("contactId", "complainedAt");


--
-- Name: emails_contactId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "emails_contactId_idx" ON public.emails USING btree ("contactId");


--
-- Name: emails_contactId_openedAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "emails_contactId_openedAt_idx" ON public.emails USING btree ("contactId", "openedAt");


--
-- Name: emails_createdAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "emails_createdAt_idx" ON public.emails USING btree ("createdAt");


--
-- Name: emails_messageId_key; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX "emails_messageId_key" ON public.emails USING btree ("messageId");


--
-- Name: emails_projectId_contactId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "emails_projectId_contactId_idx" ON public.emails USING btree ("projectId", "contactId");


--
-- Name: emails_projectId_sourceType_createdAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "emails_projectId_sourceType_createdAt_idx" ON public.emails USING btree ("projectId", "sourceType", "createdAt");


--
-- Name: emails_status_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX emails_status_idx ON public.emails USING btree (status);


--
-- Name: emails_workflowExecutionId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "emails_workflowExecutionId_idx" ON public.emails USING btree ("workflowExecutionId");


--
-- Name: emails_workflowStepExecutionId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "emails_workflowStepExecutionId_idx" ON public.emails USING btree ("workflowStepExecutionId");


--
-- Name: events_contactId_createdAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "events_contactId_createdAt_idx" ON public.events USING btree ("contactId", "createdAt");


--
-- Name: events_contactId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "events_contactId_idx" ON public.events USING btree ("contactId");


--
-- Name: events_contactId_name_createdAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "events_contactId_name_createdAt_idx" ON public.events USING btree ("contactId", name, "createdAt");


--
-- Name: events_createdAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "events_createdAt_idx" ON public.events USING btree ("createdAt");


--
-- Name: events_data_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX events_data_idx ON public.events USING gin (data);


--
-- Name: events_emailId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "events_emailId_idx" ON public.events USING btree ("emailId");


--
-- Name: events_projectId_contactId_name_createdAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "events_projectId_contactId_name_createdAt_idx" ON public.events USING btree ("projectId", "contactId", name, "createdAt");


--
-- Name: events_projectId_name_createdAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "events_projectId_name_createdAt_idx" ON public.events USING btree ("projectId", name, "createdAt");


--
-- Name: events_projectId_name_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "events_projectId_name_idx" ON public.events USING btree ("projectId", name);


--
-- Name: projects_customer_key; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX projects_customer_key ON public.projects USING btree (customer);


--
-- Name: projects_public_key; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX projects_public_key ON public.projects USING btree (public);


--
-- Name: projects_secret_key; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX projects_secret_key ON public.projects USING btree (secret);


--
-- Name: projects_subscription_key; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX projects_subscription_key ON public.projects USING btree (subscription);


--
-- Name: segment_memberships_contactId_exitedAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "segment_memberships_contactId_exitedAt_idx" ON public.segment_memberships USING btree ("contactId", "exitedAt");


--
-- Name: segment_memberships_enteredAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "segment_memberships_enteredAt_idx" ON public.segment_memberships USING btree ("enteredAt");


--
-- Name: segment_memberships_segmentId_exitedAt_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "segment_memberships_segmentId_exitedAt_idx" ON public.segment_memberships USING btree ("segmentId", "exitedAt");


--
-- Name: segments_condition_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX segments_condition_idx ON public.segments USING gin (condition);


--
-- Name: segments_projectId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "segments_projectId_idx" ON public.segments USING btree ("projectId");


--
-- Name: templates_projectId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "templates_projectId_idx" ON public.templates USING btree ("projectId");


--
-- Name: templates_projectId_type_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "templates_projectId_type_idx" ON public.templates USING btree ("projectId", type);


--
-- Name: users_email_key; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX users_email_key ON public.users USING btree (email);


--
-- Name: workflow_executions_contactId_status_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "workflow_executions_contactId_status_idx" ON public.workflow_executions USING btree ("contactId", status);


--
-- Name: workflow_executions_status_currentStepId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "workflow_executions_status_currentStepId_idx" ON public.workflow_executions USING btree (status, "currentStepId");


--
-- Name: workflow_executions_workflowId_contactId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "workflow_executions_workflowId_contactId_idx" ON public.workflow_executions USING btree ("workflowId", "contactId");


--
-- Name: workflow_executions_workflowId_status_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "workflow_executions_workflowId_status_idx" ON public.workflow_executions USING btree ("workflowId", status);


--
-- Name: workflow_step_executions_executionId_status_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "workflow_step_executions_executionId_status_idx" ON public.workflow_step_executions USING btree ("executionId", status);


--
-- Name: workflow_step_executions_scheduledFor_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "workflow_step_executions_scheduledFor_idx" ON public.workflow_step_executions USING btree ("scheduledFor");


--
-- Name: workflow_step_executions_status_scheduledFor_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "workflow_step_executions_status_scheduledFor_idx" ON public.workflow_step_executions USING btree (status, "scheduledFor");


--
-- Name: workflow_step_executions_stepId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "workflow_step_executions_stepId_idx" ON public.workflow_step_executions USING btree ("stepId");


--
-- Name: workflow_steps_templateId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "workflow_steps_templateId_idx" ON public.workflow_steps USING btree ("templateId");


--
-- Name: workflow_steps_workflowId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "workflow_steps_workflowId_idx" ON public.workflow_steps USING btree ("workflowId");


--
-- Name: workflow_transitions_fromStepId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "workflow_transitions_fromStepId_idx" ON public.workflow_transitions USING btree ("fromStepId");


--
-- Name: workflow_transitions_toStepId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "workflow_transitions_toStepId_idx" ON public.workflow_transitions USING btree ("toStepId");


--
-- Name: workflows_projectId_enabled_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "workflows_projectId_enabled_idx" ON public.workflows USING btree ("projectId", enabled);


--
-- Name: workflows_projectId_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "workflows_projectId_idx" ON public.workflows USING btree ("projectId");


--
-- Name: api_requests api_requests_projectId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.api_requests
    ADD CONSTRAINT "api_requests_projectId_fkey" FOREIGN KEY ("projectId") REFERENCES public.projects(id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: campaigns campaigns_projectId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.campaigns
    ADD CONSTRAINT "campaigns_projectId_fkey" FOREIGN KEY ("projectId") REFERENCES public.projects(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: campaigns campaigns_segmentId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.campaigns
    ADD CONSTRAINT "campaigns_segmentId_fkey" FOREIGN KEY ("segmentId") REFERENCES public.segments(id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: contacts contacts_projectId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.contacts
    ADD CONSTRAINT "contacts_projectId_fkey" FOREIGN KEY ("projectId") REFERENCES public.projects(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: domains domains_projectId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.domains
    ADD CONSTRAINT "domains_projectId_fkey" FOREIGN KEY ("projectId") REFERENCES public.projects(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: emails emails_campaignId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.emails
    ADD CONSTRAINT "emails_campaignId_fkey" FOREIGN KEY ("campaignId") REFERENCES public.campaigns(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: emails emails_contactId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.emails
    ADD CONSTRAINT "emails_contactId_fkey" FOREIGN KEY ("contactId") REFERENCES public.contacts(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: emails emails_projectId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.emails
    ADD CONSTRAINT "emails_projectId_fkey" FOREIGN KEY ("projectId") REFERENCES public.projects(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: emails emails_templateId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.emails
    ADD CONSTRAINT "emails_templateId_fkey" FOREIGN KEY ("templateId") REFERENCES public.templates(id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: emails emails_workflowExecutionId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.emails
    ADD CONSTRAINT "emails_workflowExecutionId_fkey" FOREIGN KEY ("workflowExecutionId") REFERENCES public.workflow_executions(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: emails emails_workflowStepExecutionId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.emails
    ADD CONSTRAINT "emails_workflowStepExecutionId_fkey" FOREIGN KEY ("workflowStepExecutionId") REFERENCES public.workflow_step_executions(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: events events_contactId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.events
    ADD CONSTRAINT "events_contactId_fkey" FOREIGN KEY ("contactId") REFERENCES public.contacts(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: events events_emailId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.events
    ADD CONSTRAINT "events_emailId_fkey" FOREIGN KEY ("emailId") REFERENCES public.emails(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: events events_projectId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.events
    ADD CONSTRAINT "events_projectId_fkey" FOREIGN KEY ("projectId") REFERENCES public.projects(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: memberships memberships_projectId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.memberships
    ADD CONSTRAINT "memberships_projectId_fkey" FOREIGN KEY ("projectId") REFERENCES public.projects(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: memberships memberships_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.memberships
    ADD CONSTRAINT "memberships_userId_fkey" FOREIGN KEY ("userId") REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- Name: segment_memberships segment_memberships_contactId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.segment_memberships
    ADD CONSTRAINT "segment_memberships_contactId_fkey" FOREIGN KEY ("contactId") REFERENCES public.contacts(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: segment_memberships segment_memberships_segmentId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.segment_memberships
    ADD CONSTRAINT "segment_memberships_segmentId_fkey" FOREIGN KEY ("segmentId") REFERENCES public.segments(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: segments segments_projectId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.segments
    ADD CONSTRAINT "segments_projectId_fkey" FOREIGN KEY ("projectId") REFERENCES public.projects(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: templates templates_projectId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.templates
    ADD CONSTRAINT "templates_projectId_fkey" FOREIGN KEY ("projectId") REFERENCES public.projects(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: workflow_executions workflow_executions_contactId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflow_executions
    ADD CONSTRAINT "workflow_executions_contactId_fkey" FOREIGN KEY ("contactId") REFERENCES public.contacts(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: workflow_executions workflow_executions_currentStepId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflow_executions
    ADD CONSTRAINT "workflow_executions_currentStepId_fkey" FOREIGN KEY ("currentStepId") REFERENCES public.workflow_steps(id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: workflow_executions workflow_executions_workflowId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflow_executions
    ADD CONSTRAINT "workflow_executions_workflowId_fkey" FOREIGN KEY ("workflowId") REFERENCES public.workflows(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: workflow_step_executions workflow_step_executions_executionId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflow_step_executions
    ADD CONSTRAINT "workflow_step_executions_executionId_fkey" FOREIGN KEY ("executionId") REFERENCES public.workflow_executions(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: workflow_step_executions workflow_step_executions_stepId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflow_step_executions
    ADD CONSTRAINT "workflow_step_executions_stepId_fkey" FOREIGN KEY ("stepId") REFERENCES public.workflow_steps(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: workflow_steps workflow_steps_templateId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflow_steps
    ADD CONSTRAINT "workflow_steps_templateId_fkey" FOREIGN KEY ("templateId") REFERENCES public.templates(id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: workflow_steps workflow_steps_workflowId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflow_steps
    ADD CONSTRAINT "workflow_steps_workflowId_fkey" FOREIGN KEY ("workflowId") REFERENCES public.workflows(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: workflow_transitions workflow_transitions_fromStepId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflow_transitions
    ADD CONSTRAINT "workflow_transitions_fromStepId_fkey" FOREIGN KEY ("fromStepId") REFERENCES public.workflow_steps(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: workflow_transitions workflow_transitions_toStepId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflow_transitions
    ADD CONSTRAINT "workflow_transitions_toStepId_fkey" FOREIGN KEY ("toStepId") REFERENCES public.workflow_steps(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: workflows workflows_projectId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workflows
    ADD CONSTRAINT "workflows_projectId_fkey" FOREIGN KEY ("projectId") REFERENCES public.projects(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

\unrestrict jyD0iTtwJXK57k9R2Xr1O2GvIGHJqeVJlP9xY69vlvqrH6h7OEYmgmfngc1DWIw

