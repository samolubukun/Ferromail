'use client';

import React, { useState, useEffect } from 'react';
import { Key, Globe, Copy, Check, Loader2, Save, Database, AlertCircle, Eye, EyeOff, ExternalLink, Bell } from 'lucide-react';
import SidebarLayout from '@/components/SidebarLayout';

type Tab = 'general' | 'apikeys' | 'webhooks';

const TABS: { id: Tab; label: string; icon: React.ReactNode }[] = [
  { id: 'general',  label: 'General',      icon: <Database size={14} /> },
  { id: 'apikeys',  label: 'API Keys',     icon: <Key size={14} /> },
  { id: 'webhooks', label: 'Webhooks',     icon: <Globe size={14} /> },
];

export default function Settings() {
  const [activeTab, setTab]       = useState<Tab>('general');
  const [projectName, setName]    = useState('My Workspace');
  const [projectId, setId]        = useState('');
  const [secretKey, setSecret]    = useState('');
  const [publicKey, setPublic]    = useState('');
  const [webhookUrl, setWebhook]  = useState('');
  const [webhooks, setWebhooks]   = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving]   = useState(false);
  const [copied, setCopied]       = useState<string | null>(null);
  const [showSecret, setShowSec]  = useState(false);
  const [successMsg, setSuccess]  = useState<string | null>(null);
  const [error, setError]         = useState<string | null>(null);

  useEffect(() => {
    const token  = localStorage.getItem('ferromail_token') || '';
    const projId = localStorage.getItem('ferromail_project_id') || 'proj_default';
    const email  = localStorage.getItem('ferromail_email') || '';

    setSecret(token);
    setId(projId);
    setPublic(`pk_${projId.slice(0, 8)}_${email.split('@')[0].slice(0, 6)}`);
    setIsLoading(false);
  }, []);

  const copy = (text: string, key: string) => {
    navigator.clipboard.writeText(text);
    setCopied(key);
    setTimeout(() => setCopied(null), 2000);
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSaving(true); setError(null);
    await new Promise(r => setTimeout(r, 600));
    setIsSaving(false);
    setSuccess('Settings saved successfully.');
    setTimeout(() => setSuccess(null), 3000);
  };

  const addWebhook = () => {
    if (webhookUrl.trim()) {
      setWebhooks(prev => [...prev, webhookUrl.trim()]);
      setWebhook('');
      setSuccess('Webhook endpoint added.');
      setTimeout(() => setSuccess(null), 3000);
    }
  };

  const removeWebhook = (url: string) => setWebhooks(prev => prev.filter(w => w !== url));

  if (isLoading) return (
    <SidebarLayout>
      <div style={{ padding: 80, display: 'flex', justifyContent: 'center' }}>
        <Loader2 size={28} color="#6366f1" style={{ animation: 'spin 1s linear infinite' }} />
      </div>
    </SidebarLayout>
  );

  return (
    <SidebarLayout>
      {/* Header */}
      <div style={{ marginBottom: 32 }}>
        <h2 style={{ fontFamily: 'Outfit', fontSize: 24, fontWeight: 800, color: '#f1f5f9', letterSpacing: '-0.02em', marginBottom: 4 }}>Settings</h2>
        <p style={{ fontSize: 13, color: 'var(--text-secondary)' }}>Manage your workspace, API credentials, and webhook endpoints.</p>
      </div>

      {/* Feedback */}
      {successMsg && <div className="alert-success" style={{ marginBottom: 20 }}><Check size={14} />{successMsg}</div>}
      {error      && <div className="alert-error"   style={{ marginBottom: 20 }}><AlertCircle size={14} />{error}</div>}

      {/* Layout */}
      <div style={{ display: 'grid', gridTemplateColumns: '180px 1fr', gap: 24, alignItems: 'start' }}>

        {/* Sidebar tabs */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
          {TABS.map(tab => (
            <button
              key={tab.id}
              onClick={() => setTab(tab.id)}
              style={{
                display: 'flex', alignItems: 'center', gap: 9,
                padding: '9px 12px',
                borderRadius: 9,
                fontSize: 13,
                fontWeight: activeTab === tab.id ? 600 : 500,
                color: activeTab === tab.id ? '#a5b4fc' : 'var(--text-secondary)',
                background: activeTab === tab.id ? 'rgba(99,102,241,0.1)' : 'transparent',
                border: `1px solid ${activeTab === tab.id ? 'rgba(99,102,241,0.2)' : 'transparent'}`,
                cursor: 'pointer',
                textAlign: 'left',
                transition: 'all 0.15s',
              }}
              onMouseEnter={e => { if (activeTab !== tab.id) { e.currentTarget.style.background = 'rgba(148,163,184,0.05)'; e.currentTarget.style.color = '#cbd5e1'; } }}
              onMouseLeave={e => { if (activeTab !== tab.id) { e.currentTarget.style.background = 'transparent'; e.currentTarget.style.color = 'var(--text-secondary)'; } }}
            >
              {tab.icon} {tab.label}
            </button>
          ))}
        </div>

        {/* Tab content */}
        <div>
          {/* GENERAL TAB */}
          {activeTab === 'general' && (
            <form onSubmit={handleSave} className="glass-panel" style={{ padding: 28 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 24, paddingBottom: 20, borderBottom: '1px solid var(--border-subtle)' }}>
                <div style={{ width: 32, height: 32, borderRadius: 9, background: 'rgba(99,102,241,0.1)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                  <Database size={14} color="#818cf8" />
                </div>
                <h3 style={{ fontFamily: 'Outfit', fontSize: 15, fontWeight: 700, color: '#f1f5f9' }}>General details</h3>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16, marginBottom: 24 }}>
                <div>
                  <label className="label">Workspace name</label>
                  <input type="text" value={projectName} onChange={e => setName(e.target.value)} className="input-field" />
                </div>
                <div>
                  <label className="label">Project ID</label>
                  <input type="text" value={projectId} disabled className="input-field" style={{ fontFamily: 'JetBrains Mono, monospace', fontSize: 12 }} />
                </div>
              </div>

              <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
                <button type="submit" disabled={isSaving} className="btn-primary" style={{ gap: 8 }}>
                  {isSaving
                    ? <><span style={{ width: 14, height: 14, border: '2px solid rgba(255,255,255,0.3)', borderTopColor: 'white', borderRadius: '50%', animation: 'spin 0.8s linear infinite', display: 'inline-block' }} />Saving…</>
                    : <><Save size={14} />Save Changes</>
                  }
                </button>
              </div>
            </form>
          )}

          {/* API KEYS TAB */}
          {activeTab === 'apikeys' && (
            <div className="glass-panel" style={{ padding: 28 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 24, paddingBottom: 20, borderBottom: '1px solid var(--border-subtle)' }}>
                <div style={{ width: 32, height: 32, borderRadius: 9, background: 'rgba(99,102,241,0.1)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                  <Key size={14} color="#818cf8" />
                </div>
                <div>
                  <h3 style={{ fontFamily: 'Outfit', fontSize: 15, fontWeight: 700, color: '#f1f5f9', marginBottom: 2 }}>API Keys</h3>
                  <p style={{ fontSize: 12, color: 'var(--text-muted)' }}>Use these to authenticate requests to the Ferromail API.</p>
                </div>
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
                {/* Public key */}
                <div>
                  <label className="label">Public key</label>
                  <p style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 8 }}>Safe to use in client-side code.</p>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 10, background: 'rgba(2,8,23,0.6)', border: '1px solid var(--border-default)', borderRadius: 10, padding: '10px 14px' }}>
                    <span style={{ flex: 1, fontSize: 12, fontFamily: 'JetBrains Mono, monospace', color: '#94a3b8', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{publicKey}</span>
                    <button onClick={() => copy(publicKey, 'public')} className="btn-ghost" style={{ gap: 5, fontSize: 12, flexShrink: 0 }}>
                      {copied === 'public' ? <Check size={13} color="#34d399" /> : <Copy size={13} />}
                      {copied === 'public' ? 'Copied' : 'Copy'}
                    </button>
                  </div>
                </div>

                {/* Secret key */}
                <div>
                  <label className="label">Secret key (Bearer token)</label>
                  <p style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 8 }}>Keep this private. Use in Authorization header.</p>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 10, background: 'rgba(2,8,23,0.6)', border: '1px solid var(--border-default)', borderRadius: 10, padding: '10px 14px' }}>
                    <span style={{ flex: 1, fontSize: 12, fontFamily: 'JetBrains Mono, monospace', color: '#94a3b8', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                      {showSecret ? secretKey : '•'.repeat(Math.min(secretKey.length, 48))}
                    </span>
                    <button onClick={() => setShowSec(p => !p)} className="btn-ghost" style={{ gap: 5, fontSize: 12, flexShrink: 0 }}>
                      {showSecret ? <EyeOff size={13} /> : <Eye size={13} />}
                      {showSecret ? 'Hide' : 'Show'}
                    </button>
                    <button onClick={() => copy(secretKey, 'secret')} className="btn-ghost" style={{ gap: 5, fontSize: 12, flexShrink: 0 }}>
                      {copied === 'secret' ? <Check size={13} color="#34d399" /> : <Copy size={13} />}
                      {copied === 'secret' ? 'Copied' : 'Copy'}
                    </button>
                  </div>
                </div>

                {/* Usage example */}
                <div style={{ background: 'rgba(2,8,23,0.7)', border: '1px solid var(--border-subtle)', borderRadius: 10, padding: '14px 16px' }}>
                  <div style={{ fontSize: 11, fontWeight: 700, letterSpacing: '0.07em', textTransform: 'uppercase', color: 'var(--text-muted)', marginBottom: 8 }}>Example request</div>
                  <pre style={{ fontSize: 11, fontFamily: 'JetBrains Mono, monospace', color: '#94a3b8', whiteSpace: 'pre-wrap', lineHeight: 1.7 }}>
{`curl -X POST http://localhost:3000/v1/send \\
  -H "Authorization: Bearer <your-secret-key>" \\
  -H "Content-Type: application/json" \\
  -d '{"from":"you@domain.com","to":["user@example.com"],"subject":"Hello"}'`}
                  </pre>
                </div>
              </div>
            </div>
          )}

          {/* WEBHOOKS TAB */}
          {activeTab === 'webhooks' && (
            <div className="glass-panel" style={{ padding: 28 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 24, paddingBottom: 20, borderBottom: '1px solid var(--border-subtle)' }}>
                <div style={{ width: 32, height: 32, borderRadius: 9, background: 'rgba(99,102,241,0.1)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                  <Globe size={14} color="#818cf8" />
                </div>
                <div>
                  <h3 style={{ fontFamily: 'Outfit', fontSize: 15, fontWeight: 700, color: '#f1f5f9', marginBottom: 2 }}>Webhook Endpoints</h3>
                  <p style={{ fontSize: 12, color: 'var(--text-muted)' }}>Receive real-time events (opens, clicks, bounces) at your server.</p>
                </div>
              </div>

              {/* Event types */}
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6, marginBottom: 24 }}>
                {['email.sent', 'email.delivered', 'email.opened', 'email.clicked', 'email.bounced', 'email.unsubscribed'].map(ev => (
                  <span key={ev} className="badge badge-neutral" style={{ fontFamily: 'JetBrains Mono, monospace', fontSize: 10 }}>{ev}</span>
                ))}
              </div>

              {/* Add webhook */}
              <div style={{ display: 'flex', gap: 10, marginBottom: 20 }}>
                <input
                  type="url"
                  value={webhookUrl}
                  onChange={e => setWebhook(e.target.value)}
                  placeholder="https://api.yourapp.com/webhooks/ferromail"
                  className="input-field"
                  onKeyDown={e => { if (e.key === 'Enter') { e.preventDefault(); addWebhook(); } }}
                />
                <button onClick={addWebhook} disabled={!webhookUrl.trim()} className="btn-primary" style={{ flexShrink: 0, gap: 7 }}>
                  <Bell size={13} /> Add URL
                </button>
              </div>

              {/* Webhook list */}
              {webhooks.length === 0 ? (
                <div style={{ padding: '24px', textAlign: 'center', color: 'var(--text-muted)', fontSize: 13, background: 'rgba(15,26,46,0.4)', borderRadius: 10, border: '1px dashed var(--border-subtle)' }}>
                  No webhook endpoints configured yet.
                </div>
              ) : (
                <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                  {webhooks.map((url, i) => (
                    <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 10, background: 'rgba(15,26,46,0.5)', border: '1px solid var(--border-subtle)', borderRadius: 10, padding: '10px 14px' }}>
                      <ExternalLink size={13} color="var(--text-muted)" style={{ flexShrink: 0 }} />
                      <span style={{ flex: 1, fontSize: 12, fontFamily: 'JetBrains Mono, monospace', color: '#94a3b8', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{url}</span>
                      <button onClick={() => removeWebhook(url)} className="btn-ghost" style={{ color: '#fb7185', flexShrink: 0 }}>Remove</button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </SidebarLayout>
  );
}
