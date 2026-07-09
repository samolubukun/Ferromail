'use client';

import React, { useState, useEffect } from 'react';
import { Mail, Plus, Send, Loader2, Calendar, CheckCircle2, Clock, ArrowLeft, Users, FileText, Settings2 } from 'lucide-react';
import SidebarLayout from '@/components/SidebarLayout';

interface Campaign {
  id: string;
  name: string;
  status: string;
  subject: string;
  totalRecipients: number;
  sentAt: string | null;
  createdAt: string;
}

const STATUS_MAP: Record<string, { label: string; class: string }> = {
  SENT:      { label: 'Sent',      class: 'badge badge-success' },
  SENDING:   { label: 'Sending…',  class: 'badge badge-accent' },
  SCHEDULED: { label: 'Scheduled', class: 'badge badge-info' },
  DRAFT:     { label: 'Draft',     class: 'badge badge-neutral' },
  CANCELLED: { label: 'Cancelled', class: 'badge badge-danger' },
};

const STEPS = ['Details', 'Content', 'Schedule'];

export default function Campaigns() {
  const [campaigns, setCampaigns]       = useState<Campaign[]>([]);
  const [view, setView]                 = useState<'list' | 'create'>('list');
  const [isLoading, setIsLoading]       = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError]               = useState<string | null>(null);
  const [step, setStep]                 = useState(0);

  // Step 1
  const [name, setName]               = useState('');
  const [description, setDesc]        = useState('');
  const [audienceType, setAudience]   = useState('ALL');
  const [from, setFrom]               = useState('');
  const [fromName, setFromName]       = useState('');
  const [replyTo, setReplyTo]         = useState('');
  // Step 2
  const [subject, setSubject]         = useState('');
  const [body, setBody]               = useState('');
  // Step 3
  const [sendNow, setSendNow]         = useState(true);
  const [scheduledFor, setScheduled]  = useState('');

  const getToken = () => localStorage.getItem('ferromail_token') || '';

  const fetchCampaigns = async () => {
    setIsLoading(true); setError(null);
    try {
      const res  = await fetch('http://localhost:3000/v1/campaigns', { headers: { Authorization: `Bearer ${getToken()}` } });
      const data = await res.json();
      if (res.ok) setCampaigns(data); else setError(data.error || 'Failed to load campaigns.');
    } catch { setError('Cannot connect to backend.'); }
    finally { setIsLoading(false); }
  };

  useEffect(() => { fetchCampaigns(); }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault(); setIsSubmitting(true); setError(null);
    const payload = { name, description: description || null, subject, body, from, fromName: fromName || null, replyTo: replyTo || null, audienceType, scheduledFor: sendNow ? null : scheduledFor, sendImmediately: sendNow };
    try {
      const res  = await fetch('http://localhost:3000/v1/campaigns', { method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${getToken()}` }, body: JSON.stringify(payload) });
      const data = await res.json();
      if (res.ok) { setView('list'); setStep(0); fetchCampaigns(); } else setError(data.error || 'Failed to create campaign.');
    } catch { setError('Connection error.'); }
    finally { setIsSubmitting(false); }
  };

  const resetForm = () => { setName(''); setDesc(''); setFrom(''); setFromName(''); setReplyTo(''); setSubject(''); setBody(''); setSendNow(true); setScheduled(''); setStep(0); };

  return (
    <SidebarLayout>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 32 }}>
        <div>
          <h2 style={{ fontFamily: 'Outfit', fontSize: 24, fontWeight: 800, color: '#f1f5f9', letterSpacing: '-0.02em', marginBottom: 4 }}>
            {view === 'list' ? 'Campaigns' : 'New Campaign'}
          </h2>
          <p style={{ fontSize: 13, color: 'var(--text-secondary)' }}>
            {view === 'list' ? 'Design and broadcast newsletters to your audience.' : 'Fill in the details to launch a new email broadcast.'}
          </p>
        </div>
        {view === 'list' ? (
          <button onClick={() => { setView('create'); resetForm(); }} className="btn-primary" style={{ gap: 7 }}>
            <Plus size={15} /> New Campaign
          </button>
        ) : (
          <button onClick={() => { setView('list'); resetForm(); }} className="btn-secondary" style={{ gap: 7 }}>
            <ArrowLeft size={14} /> Back to list
          </button>
        )}
      </div>

      {error && <div className="alert-error" style={{ marginBottom: 20 }}>{error}</div>}

      {/* LIST VIEW */}
      {view === 'list' && (
        isLoading ? (
          <div className="glass-panel" style={{ padding: 60, display: 'flex', justifyContent: 'center' }}>
            <Loader2 size={28} color="#6366f1" style={{ animation: 'spin 1s linear infinite' }} />
          </div>
        ) : campaigns.length === 0 ? (
          <div className="glass-panel">
            <div className="empty-state">
              <div className="empty-state-icon"><Mail size={22} /></div>
              <h3 style={{ fontFamily: 'Outfit', fontSize: 16, fontWeight: 700, color: '#f1f5f9' }}>No campaigns yet</h3>
              <p style={{ fontSize: 13, color: 'var(--text-secondary)', maxWidth: 300 }}>Create your first campaign to send newsletters or product updates to your audience.</p>
              <button onClick={() => { setView('create'); resetForm(); }} className="btn-primary" style={{ marginTop: 8, gap: 7 }}>
                <Plus size={14} /> Create Campaign
              </button>
            </div>
          </div>
        ) : (
          <div className="glass-panel" style={{ overflow: 'hidden' }}>
            <table className="data-table">
              <thead>
                <tr>
                  <th>Campaign</th>
                  <th>Status</th>
                  <th>Recipients</th>
                  <th>Sent</th>
                  <th>Created</th>
                </tr>
              </thead>
              <tbody>
                {campaigns.map(c => {
                  const status = STATUS_MAP[c.status] || { label: c.status, class: 'badge badge-neutral' };
                  return (
                    <tr key={c.id}>
                      <td>
                        <p style={{ fontWeight: 600, color: '#f1f5f9', marginBottom: 2 }}>{c.name}</p>
                        <p style={{ fontSize: 12, color: 'var(--text-muted)', maxWidth: 260, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{c.subject}</p>
                      </td>
                      <td><span className={status.class}>{status.label}</span></td>
                      <td style={{ fontFamily: 'JetBrains Mono, monospace', fontWeight: 600, color: '#e2e8f0' }}>
                        {c.totalRecipients > 0 ? c.totalRecipients.toLocaleString() : '—'}
                      </td>
                      <td style={{ fontFamily: 'JetBrains Mono, monospace', fontSize: 12, color: 'var(--text-muted)' }}>
                        {c.sentAt ? c.sentAt.split('T')[0] : '—'}
                      </td>
                      <td style={{ fontFamily: 'JetBrains Mono, monospace', fontSize: 12, color: 'var(--text-muted)' }}>
                        {c.createdAt.split('T')[0]}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )
      )}

      {/* CREATE VIEW */}
      {view === 'create' && (
        <div style={{ maxWidth: 760 }}>
          {/* Step progress */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 0, marginBottom: 32 }}>
            {STEPS.map((s, i) => (
              <React.Fragment key={s}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <div style={{
                    width: 28, height: 28, borderRadius: '50%',
                    background: i <= step ? '#6366f1' : 'var(--bg-elevated)',
                    border: `2px solid ${i <= step ? '#6366f1' : 'var(--border-default)'}`,
                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                    fontSize: 12, fontWeight: 700,
                    color: i <= step ? 'white' : 'var(--text-muted)',
                    transition: 'all 0.2s',
                  }}>
                    {i < step ? <CheckCircle2 size={14} /> : i + 1}
                  </div>
                  <span style={{ fontSize: 13, fontWeight: i === step ? 600 : 400, color: i <= step ? '#e2e8f0' : 'var(--text-muted)' }}>{s}</span>
                </div>
                {i < STEPS.length - 1 && (
                  <div style={{ flex: 1, height: 1, background: i < step ? '#6366f1' : 'var(--border-subtle)', margin: '0 12px', transition: 'background 0.3s' }} />
                )}
              </React.Fragment>
            ))}
          </div>

          <form onSubmit={handleSubmit}>
            {/* STEP 0: Details */}
            {step === 0 && (
              <div className="glass-panel" style={{ padding: 28 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 20 }}>
                  <div style={{ width: 32, height: 32, borderRadius: 9, background: 'rgba(99,102,241,0.1)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                    <FileText size={14} color="#818cf8" />
                  </div>
                  <h3 style={{ fontFamily: 'Outfit', fontSize: 15, fontWeight: 700, color: '#f1f5f9' }}>Campaign Details</h3>
                </div>

                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16, marginBottom: 16 }}>
                  <div>
                    <label className="label">Campaign name *</label>
                    <input type="text" value={name} onChange={e => setName(e.target.value)} placeholder="Summer Promo 2026" required className="input-field" />
                  </div>
                  <div>
                    <label className="label">Audience</label>
                    <select value={audienceType} onChange={e => setAudience(e.target.value)} className="input-field">
                      <option value="ALL">All subscribed contacts</option>
                      <option value="SEGMENT">Custom segment</option>
                    </select>
                  </div>
                </div>

                <div style={{ marginBottom: 16 }}>
                  <label className="label">Description</label>
                  <input type="text" value={description} onChange={e => setDesc(e.target.value)} placeholder="Optional short description" className="input-field" />
                </div>

                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: 16 }}>
                  <div>
                    <label className="label">From address *</label>
                    <input type="email" value={from} onChange={e => setFrom(e.target.value)} placeholder="news@domain.com" required className="input-field" />
                  </div>
                  <div>
                    <label className="label">Sender name</label>
                    <input type="text" value={fromName} onChange={e => setFromName(e.target.value)} placeholder="Acme Team" className="input-field" />
                  </div>
                  <div>
                    <label className="label">Reply-to</label>
                    <input type="email" value={replyTo} onChange={e => setReplyTo(e.target.value)} placeholder="support@domain.com" className="input-field" />
                  </div>
                </div>

                <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: 24 }}>
                  <button type="button" onClick={() => setStep(1)} disabled={!name || !from} className="btn-primary">
                    Next: Content →
                  </button>
                </div>
              </div>
            )}

            {/* STEP 1: Content */}
            {step === 1 && (
              <div className="glass-panel" style={{ padding: 28 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 20 }}>
                  <div style={{ width: 32, height: 32, borderRadius: 9, background: 'rgba(99,102,241,0.1)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                    <Mail size={14} color="#818cf8" />
                  </div>
                  <h3 style={{ fontFamily: 'Outfit', fontSize: 15, fontWeight: 700, color: '#f1f5f9' }}>Email Content</h3>
                </div>

                <div style={{ marginBottom: 16 }}>
                  <label className="label">Subject line *</label>
                  <input type="text" value={subject} onChange={e => setSubject(e.target.value)} placeholder="Don't miss our summer sale!" required className="input-field" />
                </div>

                <div>
                  <label className="label">HTML body *</label>
                  <p style={{ fontSize: 11, color: 'var(--text-muted)', marginBottom: 6 }}>
                    Supports template variables: <code style={{ background: 'var(--bg-elevated)', padding: '1px 5px', borderRadius: 4, fontFamily: 'JetBrains Mono, monospace' }}>{'{{firstName ?? Reader}}'}</code>
                  </p>
                  <textarea
                    value={body}
                    onChange={e => setBody(e.target.value)}
                    required
                    rows={12}
                    placeholder={'<h1>Hello {{firstName ?? Reader}}!</h1>\n<p>Your content here.</p>'}
                    className="input-field"
                    style={{ fontFamily: 'JetBrains Mono, monospace', fontSize: 12, resize: 'vertical' }}
                  />
                </div>

                <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: 24 }}>
                  <button type="button" onClick={() => setStep(0)} className="btn-secondary">← Back</button>
                  <button type="button" onClick={() => setStep(2)} disabled={!subject || !body} className="btn-primary">Next: Schedule →</button>
                </div>
              </div>
            )}

            {/* STEP 2: Schedule */}
            {step === 2 && (
              <div className="glass-panel" style={{ padding: 28 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 20 }}>
                  <div style={{ width: 32, height: 32, borderRadius: 9, background: 'rgba(99,102,241,0.1)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                    <Settings2 size={14} color="#818cf8" />
                  </div>
                  <h3 style={{ fontFamily: 'Outfit', fontSize: 15, fontWeight: 700, color: '#f1f5f9' }}>Delivery Schedule</h3>
                </div>

                {/* Summary */}
                <div style={{ background: 'var(--bg-elevated)', border: '1px solid var(--border-subtle)', borderRadius: 12, padding: '14px 18px', marginBottom: 24 }}>
                  <div style={{ fontSize: 11, fontWeight: 700, letterSpacing: '0.07em', textTransform: 'uppercase', color: 'var(--text-muted)', marginBottom: 10 }}>Campaign Summary</div>
                  <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '6px 24px' }}>
                    {[['Name', name], ['From', from], ['Subject', subject], ['Audience', audienceType === 'ALL' ? 'All contacts' : 'Segment']].map(([k, v]) => (
                      <div key={k} style={{ display: 'flex', gap: 6, fontSize: 12 }}>
                        <span style={{ color: 'var(--text-muted)', minWidth: 60 }}>{k}:</span>
                        <span style={{ color: '#e2e8f0', fontWeight: 500, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{v}</span>
                      </div>
                    ))}
                  </div>
                </div>

                <div style={{ display: 'flex', flexDirection: 'column', gap: 12, marginBottom: 24 }}>
                  {[
                    { id: 'now', label: 'Send immediately', sub: 'Deliver to all recipients right now', val: true },
                    { id: 'sched', label: 'Schedule for later', sub: 'Pick a specific date and time', val: false },
                  ].map(opt => (
                    <label key={opt.id} htmlFor={opt.id} style={{ display: 'flex', alignItems: 'flex-start', gap: 12, padding: '14px 16px', borderRadius: 10, border: `1px solid ${sendNow === opt.val ? 'rgba(99,102,241,0.35)' : 'var(--border-default)'}`, background: sendNow === opt.val ? 'rgba(99,102,241,0.06)' : 'var(--bg-elevated)', cursor: 'pointer', transition: 'all 0.15s' }}>
                      <input type="radio" id={opt.id} name="sendTime" checked={sendNow === opt.val} onChange={() => setSendNow(opt.val)} style={{ marginTop: 2 }} />
                      <div>
                        <div style={{ fontSize: 13, fontWeight: 600, color: '#e2e8f0', marginBottom: 2 }}>{opt.label}</div>
                        <div style={{ fontSize: 12, color: 'var(--text-muted)' }}>{opt.sub}</div>
                      </div>
                    </label>
                  ))}
                </div>

                {!sendNow && (
                  <div style={{ marginBottom: 24 }}>
                    <label className="label">Schedule date & time</label>
                    <input type="datetime-local" value={scheduledFor} onChange={e => setScheduled(e.target.value)} required={!sendNow} className="input-field" />
                  </div>
                )}

                <div style={{ display: 'flex', justifyContent: 'space-between', borderTop: '1px solid var(--border-subtle)', paddingTop: 20 }}>
                  <button type="button" onClick={() => setStep(1)} className="btn-secondary">← Back</button>
                  <button type="submit" disabled={isSubmitting || (!sendNow && !scheduledFor)} className="btn-primary" style={{ gap: 8 }}>
                    {isSubmitting ? (
                      <><span style={{ width: 14, height: 14, border: '2px solid rgba(255,255,255,0.3)', borderTopColor: 'white', borderRadius: '50%', animation: 'spin 0.8s linear infinite', display: 'inline-block' }} /> Launching...</>
                    ) : (
                      <><Send size={14} /> {sendNow ? 'Launch Campaign' : 'Schedule Campaign'}</>
                    )}
                  </button>
                </div>
              </div>
            )}
          </form>
        </div>
      )}
    </SidebarLayout>
  );
}
