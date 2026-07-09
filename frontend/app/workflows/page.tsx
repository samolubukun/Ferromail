'use client';

import React, { useState, useEffect } from 'react';
import { Zap, Plus, GitFork, Clock, Mail, Play, Pause, Loader2, CheckCircle2, ArrowLeft, AlertCircle } from 'lucide-react';
import SidebarLayout from '@/components/SidebarLayout';

interface WorkflowRecord {
  id: string;
  name: string;
  description: string | null;
  enabled: boolean;
  triggerType: string;
}

const TRIGGER_ICONS: Record<string, React.ReactNode> = {
  EVENT:   <GitFork size={14} color="#818cf8" />,
  DELAY:   <Clock size={14} color="#fbbf24" />,
  DEFAULT: <Zap size={14} color="#818cf8" />,
};

export default function Workflows() {
  const [workflows, setWorkflows]       = useState<WorkflowRecord[]>([]);
  const [view, setView]                 = useState<'list' | 'create'>('list');
  const [isLoading, setIsLoading]       = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError]               = useState<string | null>(null);

  const [name, setName]               = useState('');
  const [description, setDesc]        = useState('');
  const [triggerType, setTrigger]     = useState('EVENT');
  const [eventName, setEvent]         = useState('user.signup');
  const [delayAmount, setDelay]       = useState(24);
  const [delayUnit, setUnit]          = useState('hours');
  const [emailSubject, setSubject]    = useState('Welcome! You\'re in.');
  const [emailBody, setBody]          = useState('<p>We\'re glad to have you. Let us know if you need anything.</p>');

  const token = () => localStorage.getItem('ferromail_token') || '';

  const fetchWorkflows = async () => {
    setIsLoading(true); setError(null);
    try {
      const res  = await fetch('http://localhost:3000/v1/workflows', { headers: { Authorization: `Bearer ${token()}` } });
      const data = await res.json();
      if (res.ok) setWorkflows(data); else setError(data.error || 'Failed to load workflows.');
    } catch { setError('Cannot connect to backend.'); }
    finally { setIsLoading(false); }
  };

  useEffect(() => { fetchWorkflows(); }, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault(); setIsSubmitting(true); setError(null);
    const tid = 'step_trigger', did = 'step_delay', eid = 'step_email';
    const payload = {
      name, description: description || null, triggerType,
      triggerConfig: triggerType === 'EVENT' ? { eventName } : null,
      allowReentry: false,
      steps: [
        { id: tid, type: 'TRIGGER',    name: 'Trigger',    position: { x: 250, y: 50 },  config: { eventName } },
        { id: did, type: 'DELAY',      name: 'Wait',       position: { x: 250, y: 180 }, config: { amount: delayAmount, unit: delayUnit } },
        { id: eid, type: 'SEND_EMAIL', name: 'Send Email', position: { x: 250, y: 310 }, config: { subject: emailSubject, body: emailBody }, templateId: null },
      ],
      transitions: [
        { fromStepId: tid, toStepId: did, condition: null },
        { fromStepId: did, toStepId: eid, condition: null },
      ],
    };
    try {
      const res  = await fetch('http://localhost:3000/v1/workflows', { method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token()}` }, body: JSON.stringify(payload) });
      const data = await res.json();
      if (res.ok && data.success) { setView('list'); fetchWorkflows(); }
      else setError(data.error || 'Failed to create workflow.');
    } catch { setError('Connection error.'); }
    finally { setIsSubmitting(false); }
  };

  return (
    <SidebarLayout>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', marginBottom: 32 }}>
        <div>
          <h2 style={{ fontFamily: 'Outfit', fontSize: 24, fontWeight: 800, color: '#f1f5f9', letterSpacing: '-0.02em', marginBottom: 4 }}>
            {view === 'list' ? 'Workflows' : 'New Workflow'}
          </h2>
          <p style={{ fontSize: 13, color: 'var(--text-secondary)' }}>
            {view === 'list' ? 'Automated email journeys triggered by user events.' : 'Configure a trigger → wait → send email pipeline.'}
          </p>
        </div>
        {view === 'list' ? (
          <button onClick={() => setView('create')} className="btn-primary" style={{ gap: 7 }}>
            <Plus size={15} /> New Workflow
          </button>
        ) : (
          <button onClick={() => setView('list')} className="btn-secondary" style={{ gap: 7 }}>
            <ArrowLeft size={14} /> Back
          </button>
        )}
      </div>

      {error && <div className="alert-error" style={{ marginBottom: 20 }}><AlertCircle size={14} />{error}</div>}

      {/* LIST VIEW */}
      {view === 'list' && (
        isLoading ? (
          <div className="glass-panel" style={{ padding: 60, display: 'flex', justifyContent: 'center' }}>
            <Loader2 size={28} color="#6366f1" style={{ animation: 'spin 1s linear infinite' }} />
          </div>
        ) : workflows.length === 0 ? (
          <div className="glass-panel">
            <div className="empty-state">
              <div className="empty-state-icon"><Zap size={22} /></div>
              <h3 style={{ fontFamily: 'Outfit', fontSize: 16, fontWeight: 700, color: '#f1f5f9' }}>No workflows yet</h3>
              <p style={{ fontSize: 13, color: 'var(--text-secondary)', maxWidth: 300 }}>Set up onboarding flows, dunning sequences, or any event-triggered email journey.</p>
              <button onClick={() => setView('create')} className="btn-primary" style={{ marginTop: 8, gap: 7 }}>
                <Plus size={14} /> Create Workflow
              </button>
            </div>
          </div>
        ) : (
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px,1fr))', gap: 16 }}>
            {workflows.map(flow => (
              <div key={flow.id} className="glass-panel" style={{ padding: '20px 22px', display: 'flex', flexDirection: 'column', gap: 12 }}>
                <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', gap: 10 }}>
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <h4 style={{ fontFamily: 'Outfit', fontWeight: 700, fontSize: 14, color: '#f1f5f9', marginBottom: 4, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                      {flow.name}
                    </h4>
                    <p style={{ fontSize: 12, color: 'var(--text-muted)', lineHeight: 1.5, display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden' }}>
                      {flow.description || 'No description.'}
                    </p>
                  </div>
                  <span className={flow.enabled ? 'badge badge-success' : 'badge badge-neutral'} style={{ flexShrink: 0 }}>
                    {flow.enabled ? <><Play size={9} />Running</> : <><Pause size={9} />Paused</>}
                  </span>
                </div>

                <div style={{ display: 'flex', gap: 6 }}>
                  {/* Steps visual */}
                  {[
                    { icon: TRIGGER_ICONS[flow.triggerType] || TRIGGER_ICONS.DEFAULT, label: flow.triggerType, color: '#818cf8' },
                    { icon: <Clock size={11} color="#fbbf24" />, label: 'Wait',  color: '#fbbf24' },
                    { icon: <Mail  size={11} color="#34d399" />, label: 'Email', color: '#34d399' },
                  ].map((step, i) => (
                    <React.Fragment key={i}>
                      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 3 }}>
                        <div style={{ width: 28, height: 28, borderRadius: 8, background: `${step.color}18`, border: `1px solid ${step.color}33`, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                          {step.icon}
                        </div>
                        <span style={{ fontSize: 9, color: step.color, fontWeight: 700, textTransform: 'uppercase', fontFamily: 'JetBrains Mono, monospace', letterSpacing: '0.04em' }}>{step.label}</span>
                      </div>
                      {i < 2 && <div style={{ flex: 1, height: 1, background: 'var(--border-subtle)', marginTop: 14 }} />}
                    </React.Fragment>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )
      )}

      {/* CREATE VIEW */}
      {view === 'create' && (
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 320px', gap: 20, alignItems: 'start' }}>

          {/* Config Form */}
          <form onSubmit={handleCreate} className="glass-panel" style={{ padding: 28 }}>
            <h3 style={{ fontFamily: 'Outfit', fontSize: 15, fontWeight: 700, color: '#f1f5f9', marginBottom: 20, paddingBottom: 16, borderBottom: '1px solid var(--border-subtle)' }}>
              Workflow Configuration
            </h3>

            <div style={{ display: 'flex', flexDirection: 'column', gap: 14, marginBottom: 20 }}>
              <div>
                <label className="label">Workflow name *</label>
                <input type="text" value={name} onChange={e => setName(e.target.value)} placeholder="Onboarding Sequence" required className="input-field" />
              </div>
              <div>
                <label className="label">Description</label>
                <input type="text" value={description} onChange={e => setDesc(e.target.value)} placeholder="Triggered on new user signup" className="input-field" />
              </div>
            </div>

            {/* Step 1 */}
            <div style={{ background: 'rgba(99,102,241,0.05)', border: '1px solid rgba(99,102,241,0.15)', borderRadius: 10, padding: '14px 16px', marginBottom: 10 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 7, marginBottom: 12 }}>
                <div style={{ width: 22, height: 22, borderRadius: 6, background: 'rgba(99,102,241,0.15)', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 10, fontWeight: 700, color: '#818cf8', fontFamily: 'Outfit' }}>1</div>
                <span style={{ fontSize: 12, fontWeight: 700, color: '#818cf8', textTransform: 'uppercase', letterSpacing: '0.07em' }}>Trigger Event</span>
              </div>
              <div>
                <label className="label">Event name</label>
                <input type="text" value={eventName} onChange={e => setEvent(e.target.value)} placeholder="user.signup" required className="input-field" style={{ fontFamily: 'JetBrains Mono, monospace' }} />
              </div>
            </div>

            {/* Step 2 */}
            <div style={{ background: 'rgba(245,158,11,0.05)', border: '1px solid rgba(245,158,11,0.15)', borderRadius: 10, padding: '14px 16px', marginBottom: 10 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 7, marginBottom: 12 }}>
                <div style={{ width: 22, height: 22, borderRadius: 6, background: 'rgba(245,158,11,0.15)', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 10, fontWeight: 700, color: '#fbbf24', fontFamily: 'Outfit' }}>2</div>
                <span style={{ fontSize: 12, fontWeight: 700, color: '#fbbf24', textTransform: 'uppercase', letterSpacing: '0.07em' }}>Wait Timer</span>
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr', gap: 10 }}>
                <div>
                  <label className="label">Duration</label>
                  <input type="number" value={delayAmount} onChange={e => setDelay(parseInt(e.target.value) || 0)} min="1" required className="input-field" />
                </div>
                <div>
                  <label className="label">Unit</label>
                  <select value={delayUnit} onChange={e => setUnit(e.target.value)} className="input-field">
                    <option value="seconds">Seconds</option>
                    <option value="minutes">Minutes</option>
                    <option value="hours">Hours</option>
                    <option value="days">Days</option>
                  </select>
                </div>
              </div>
            </div>

            {/* Step 3 */}
            <div style={{ background: 'rgba(16,185,129,0.05)', border: '1px solid rgba(16,185,129,0.15)', borderRadius: 10, padding: '14px 16px', marginBottom: 20 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 7, marginBottom: 12 }}>
                <div style={{ width: 22, height: 22, borderRadius: 6, background: 'rgba(16,185,129,0.15)', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 10, fontWeight: 700, color: '#34d399', fontFamily: 'Outfit' }}>3</div>
                <span style={{ fontSize: 12, fontWeight: 700, color: '#34d399', textTransform: 'uppercase', letterSpacing: '0.07em' }}>Send Email</span>
              </div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                <div>
                  <label className="label">Subject</label>
                  <input type="text" value={emailSubject} onChange={e => setSubject(e.target.value)} required className="input-field" />
                </div>
                <div>
                  <label className="label">Body (HTML)</label>
                  <textarea value={emailBody} onChange={e => setBody(e.target.value)} rows={4} className="input-field" style={{ fontFamily: 'JetBrains Mono, monospace', fontSize: 12, resize: 'vertical' }} />
                </div>
              </div>
            </div>

            <div style={{ display: 'flex', gap: 10, justifyContent: 'flex-end' }}>
              <button type="button" onClick={() => setView('list')} className="btn-secondary">Cancel</button>
              <button type="submit" disabled={isSubmitting || !name} className="btn-primary" style={{ gap: 8 }}>
                {isSubmitting
                  ? <><span style={{ width: 14, height: 14, border: '2px solid rgba(255,255,255,0.3)', borderTopColor: 'white', borderRadius: '50%', animation: 'spin 0.8s linear infinite', display: 'inline-block' }} />Saving…</>
                  : <><CheckCircle2 size={14} />Save Workflow</>
                }
              </button>
            </div>
          </form>

          {/* Live Preview */}
          <div className="glass-panel" style={{ padding: 22, position: 'sticky', top: 24 }}>
            <div style={{ fontSize: 11, fontWeight: 700, letterSpacing: '0.1em', textTransform: 'uppercase', color: 'var(--text-muted)', marginBottom: 18, fontFamily: 'JetBrains Mono, monospace' }}>
              Flow Preview
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 0 }}>
              {[
                { bg: 'rgba(99,102,241,0.08)', border: 'rgba(99,102,241,0.25)', icon: <GitFork size={16} color="#818cf8" />, label: 'TRIGGER', title: eventName || 'user.event', color: '#818cf8' },
                { bg: 'rgba(245,158,11,0.08)', border: 'rgba(245,158,11,0.2)', icon: <Clock size={16} color="#fbbf24" />, label: 'WAIT', title: `${delayAmount} ${delayUnit}`, color: '#fbbf24' },
                { bg: 'rgba(16,185,129,0.08)', border: 'rgba(16,185,129,0.2)', icon: <Mail size={16} color="#34d399" />, label: 'SEND EMAIL', title: emailSubject || 'Email subject', color: '#34d399' },
              ].map((node, i) => (
                <React.Fragment key={i}>
                  <div style={{ width: '100%', background: node.bg, border: `1px solid ${node.border}`, borderRadius: 10, padding: '12px 14px', display: 'flex', alignItems: 'center', gap: 10 }}>
                    <div style={{ width: 32, height: 32, borderRadius: 9, background: `${node.color}18`, display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}>
                      {node.icon}
                    </div>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <div style={{ fontSize: 9, fontWeight: 700, letterSpacing: '0.08em', color: node.color, textTransform: 'uppercase', fontFamily: 'JetBrains Mono, monospace', marginBottom: 2 }}>{node.label}</div>
                      <div style={{ fontSize: 12, fontWeight: 600, color: '#e2e8f0', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{node.title}</div>
                    </div>
                  </div>
                  {i < 2 && (
                    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '4px 0' }}>
                      <div style={{ width: 1.5, height: 20, background: 'linear-gradient(to bottom, transparent, rgba(99,102,241,0.4), transparent)' }} />
                      <div style={{ width: 6, height: 6, borderRadius: '50%', background: '#6366f1', opacity: 0.5 }} />
                      <div style={{ width: 1.5, height: 20, background: 'linear-gradient(to bottom, transparent, rgba(99,102,241,0.4), transparent)' }} />
                    </div>
                  )}
                </React.Fragment>
              ))}
            </div>
          </div>
        </div>
      )}
    </SidebarLayout>
  );
}
