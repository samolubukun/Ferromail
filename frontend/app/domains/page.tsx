'use client';

import React, { useState, useEffect } from 'react';
import { Globe, Plus, CheckCircle2, XCircle, ShieldCheck, Copy, RefreshCw, Trash2, Loader2, AlertCircle, Check } from 'lucide-react';
import SidebarLayout from '@/components/SidebarLayout';

interface DomainRecord {
  id: string;
  domain: string;
  verified: boolean;
  dkimTokens: string[];
}

export default function Domains() {
  const [domains, setDomains]       = useState<DomainRecord[]>([]);
  const [newDomain, setNew]         = useState('');
  const [isLoading, setIsLoading]   = useState(true);
  const [isAdding, setIsAdding]     = useState(false);
  const [error, setError]           = useState<string | null>(null);
  const [copiedToken, setCopied]    = useState<string | null>(null);
  const [deleteConfirm, setDelConf] = useState<string | null>(null);
  const [verifying, setVerifying]   = useState<string | null>(null);

  const token = () => localStorage.getItem('ferromail_token') || '';

  const fetchDomains = async () => {
    setIsLoading(true); setError(null);
    try {
      const res  = await fetch('http://localhost:3000/v1/domains', { headers: { Authorization: `Bearer ${token()}` } });
      const data = await res.json();
      if (res.ok) setDomains(data); else setError(data.error || 'Failed to load domains.');
    } catch { setError('Cannot connect to backend.'); }
    finally { setIsLoading(false); }
  };

  useEffect(() => { fetchDomains(); }, []);

  const handleAdd = async (e: React.FormEvent) => {
    e.preventDefault(); setIsAdding(true); setError(null);
    try {
      const res  = await fetch('http://localhost:3000/v1/domains', { method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token()}` }, body: JSON.stringify({ domain: newDomain }) });
      const data = await res.json();
      if (res.ok) { setNew(''); fetchDomains(); } else setError(data.error || 'Failed to add domain.');
    } catch { setError('Connection error.'); }
    finally { setIsAdding(false); }
  };

  const handleVerify = async (id: string) => {
    setVerifying(id); setError(null);
    try {
      const res = await fetch(`http://localhost:3000/v1/domains/${id}/verify`, { method: 'POST', headers: { Authorization: `Bearer ${token()}` } });
      if (res.ok) fetchDomains(); else { const d = await res.json(); setError(d.error || 'Verification failed.'); }
    } catch { setError('Connection error.'); }
    finally { setVerifying(null); }
  };

  const handleDelete = async (id: string) => {
    setDelConf(null); setError(null);
    try {
      const res = await fetch(`http://localhost:3000/v1/domains/${id}`, { method: 'DELETE', headers: { Authorization: `Bearer ${token()}` } });
      if (res.ok) fetchDomains(); else { const d = await res.json(); setError(d.error || 'Delete failed.'); }
    } catch { setError('Connection error.'); }
  };

  const copy = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(text);
    setTimeout(() => setCopied(null), 2000);
  };

  const verifiedCount = domains.filter(d => d.verified).length;

  return (
    <SidebarLayout>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', marginBottom: 28 }}>
        <div>
          <h2 style={{ fontFamily: 'Outfit', fontSize: 24, fontWeight: 800, color: '#f1f5f9', letterSpacing: '-0.02em', marginBottom: 4 }}>Sending Domains</h2>
          <p style={{ fontSize: 13, color: 'var(--text-secondary)' }}>Configure DKIM/SPF DNS records to enable authenticated outbound email delivery.</p>
        </div>
        <div style={{ display: 'flex', gap: 10, alignItems: 'center' }}>
          {!isLoading && <span style={{ fontSize: 12, color: 'var(--text-muted)', background: 'var(--bg-elevated)', border: '1px solid var(--border-subtle)', borderRadius: 8, padding: '5px 12px' }}>
            <span style={{ color: '#34d399', fontWeight: 700 }}>{verifiedCount}</span> / {domains.length} verified
          </span>}
        </div>
      </div>

      {error && <div className="alert-error" style={{ marginBottom: 20 }}><AlertCircle size={14} />{error}</div>}

      <div style={{ display: 'grid', gridTemplateColumns: '280px 1fr', gap: 20, alignItems: 'start' }}>

        {/* Add Domain */}
        <div className="glass-panel" style={{ padding: '20px 22px', position: 'sticky', top: 24 }}>
          <h3 style={{ fontFamily: 'Outfit', fontSize: 14, fontWeight: 700, color: '#f1f5f9', marginBottom: 16, display: 'flex', alignItems: 'center', gap: 7 }}>
            <Plus size={14} color="#818cf8" /> Add Domain
          </h3>
          <form onSubmit={handleAdd} style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            <div>
              <label className="label">Domain name</label>
              <input
                type="text"
                value={newDomain}
                onChange={e => setNew(e.target.value)}
                placeholder="example.com"
                required
                className="input-field"
              />
            </div>
            <button type="submit" disabled={isAdding} className="btn-primary" style={{ width: '100%', justifyContent: 'center' }}>
              {isAdding
                ? <><span style={{ width: 14, height: 14, border: '2px solid rgba(255,255,255,0.3)', borderTopColor: 'white', borderRadius: '50%', animation: 'spin 0.8s linear infinite', display: 'inline-block' }} />Adding…</>
                : <><Globe size={13} />Add Domain</>
              }
            </button>
          </form>

          <div style={{ marginTop: 20, padding: '14px 16px', background: 'rgba(99,102,241,0.06)', border: '1px solid rgba(99,102,241,0.15)', borderRadius: 10 }}>
            <div style={{ fontSize: 11, fontWeight: 700, color: '#818cf8', textTransform: 'uppercase', letterSpacing: '0.08em', marginBottom: 8 }}>How it works</div>
            <ol style={{ paddingLeft: 14, display: 'flex', flexDirection: 'column', gap: 6 }}>
              {['Add your domain', 'Copy the CNAME records', 'Add them to your DNS provider', 'Click Verify to confirm'].map((s, i) => (
                <li key={i} style={{ fontSize: 12, color: 'var(--text-secondary)', lineHeight: 1.5 }}>{s}</li>
              ))}
            </ol>
          </div>
        </div>

        {/* Domains list */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
          {isLoading ? (
            <div className="glass-panel" style={{ padding: 60, display: 'flex', justifyContent: 'center' }}>
              <Loader2 size={28} color="#6366f1" style={{ animation: 'spin 1s linear infinite' }} />
            </div>
          ) : domains.length === 0 ? (
            <div className="glass-panel">
              <div className="empty-state">
                <div className="empty-state-icon"><Globe size={22} /></div>
                <h3 style={{ fontFamily: 'Outfit', fontSize: 16, fontWeight: 700, color: '#f1f5f9' }}>No domains configured</h3>
                <p style={{ fontSize: 13, color: 'var(--text-secondary)', maxWidth: 300 }}>Add your first sending domain to begin delivering authenticated email.</p>
              </div>
            </div>
          ) : domains.map(dom => (
            <div
              key={dom.id}
              className="glass-panel"
              style={{
                padding: '20px 24px',
                borderColor: dom.verified ? 'rgba(16,185,129,0.2)' : 'var(--border-default)',
                boxShadow: dom.verified ? '0 0 20px rgba(16,185,129,0.06)' : undefined,
              }}
            >
              {/* Domain header */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                  <div style={{
                    width: 36, height: 36, borderRadius: 10,
                    background: dom.verified ? 'rgba(16,185,129,0.1)' : 'rgba(245,158,11,0.08)',
                    border: `1px solid ${dom.verified ? 'rgba(16,185,129,0.2)' : 'rgba(245,158,11,0.2)'}`,
                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                  }}>
                    {dom.verified
                      ? <ShieldCheck size={16} color="#34d399" />
                      : <Globe size={16} color="#fbbf24" />
                    }
                  </div>
                  <div>
                    <h4 style={{ fontFamily: 'Outfit', fontSize: 15, fontWeight: 700, color: '#f1f5f9', marginBottom: 2 }}>{dom.domain}</h4>
                    <p style={{ fontSize: 11, color: 'var(--text-muted)', fontFamily: 'JetBrains Mono, monospace' }}>{dom.id.slice(0, 16)}…</p>
                  </div>
                </div>

                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span className={dom.verified ? 'badge badge-success' : 'badge badge-warning'}>
                    {dom.verified ? <><CheckCircle2 size={10} /> Verified</> : <><XCircle size={10} /> Pending DNS</>}
                  </span>

                  {!dom.verified && (
                    <button
                      onClick={() => handleVerify(dom.id)}
                      disabled={verifying === dom.id}
                      className="btn-secondary"
                      style={{ padding: '5px 12px', fontSize: 12, gap: 6 }}
                      title="Check DNS records"
                    >
                      {verifying === dom.id
                        ? <Loader2 size={12} style={{ animation: 'spin 1s linear infinite' }} />
                        : <RefreshCw size={12} />
                      }
                      Verify
                    </button>
                  )}

                  {deleteConfirm === dom.id ? (
                    <div style={{ display: 'flex', gap: 6 }}>
                      <button onClick={() => handleDelete(dom.id)} className="btn-danger" style={{ padding: '5px 12px', fontSize: 12 }}>Confirm</button>
                      <button onClick={() => setDelConf(null)} className="btn-ghost" style={{ padding: '5px 10px', fontSize: 12 }}>Cancel</button>
                    </div>
                  ) : (
                    <button onClick={() => setDelConf(dom.id)} className="btn-ghost">
                      <Trash2 size={14} />
                    </button>
                  )}
                </div>
              </div>

              {/* DNS Records */}
              {true && (
                <div>
                  <div style={{ fontSize: 11, fontWeight: 700, letterSpacing: '0.07em', textTransform: 'uppercase', color: 'var(--text-muted)', marginBottom: 10 }}>
                    DNS Records to configure
                  </div>
                  <div style={{ background: 'rgba(2,8,23,0.5)', border: '1px solid var(--border-subtle)', borderRadius: 10, overflow: 'hidden' }}>
                    {/* Table header */}
                    <div style={{ display: 'grid', gridTemplateColumns: '80px 150px 1fr 85px', padding: '8px 14px', background: 'rgba(15,26,46,0.6)', borderBottom: '1px solid var(--border-subtle)', gap: 12 }}>
                      {['Type', 'Host / Name', 'Value / Point To', ''].map(h => (
                        <div key={h} style={{ fontSize: 10, fontWeight: 700, letterSpacing: '0.07em', textTransform: 'uppercase', color: 'var(--text-muted)' }}>{h}</div>
                      ))}
                    </div>
                    {[
                      { type: 'TXT', host: 'ferromail-verify', value: `ferromail-verification=${dom.id}` },
                      ...((dom.dkimTokens as string[]) || []).map(t => ({
                        type: 'CNAME',
                        host: t.split('.').slice(0, 2).join('.'),
                        value: 'dkim.ferromail.com'
                      }))
                    ].map((rec, i, arr) => (
                      <div key={i} style={{ display: 'grid', gridTemplateColumns: '80px 150px 1fr 85px', padding: '10px 14px', borderBottom: i < arr.length - 1 ? '1px solid var(--border-subtle)' : 'none', alignItems: 'center', gap: 12 }}>
                        <span style={{
                          fontSize: 10,
                          background: rec.type === 'TXT' ? 'rgba(16,185,129,0.12)' : 'rgba(99,102,241,0.12)',
                          color: rec.type === 'TXT' ? '#34d399' : '#818cf8',
                          padding: '2px 8px', borderRadius: 5, fontWeight: 700, fontFamily: 'JetBrains Mono, monospace', width: 'fit-content'
                        }}>{rec.type}</span>
                        <span style={{ fontSize: 12, fontFamily: 'JetBrains Mono, monospace', color: '#e2e8f0', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{rec.host}</span>
                        <span style={{ fontSize: 12, fontFamily: 'JetBrains Mono, monospace', color: '#94a3b8', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }} title={rec.value}>{rec.value}</span>
                        <button
                          onClick={() => copy(rec.value)}
                          className="btn-ghost"
                          style={{ justifySelf: 'end', gap: 4, fontSize: 11 }}
                        >
                          {copiedToken === rec.value ? <Check size={12} color="#34d399" /> : <Copy size={12} />}
                          {copiedToken === rec.value ? 'Copied' : 'Copy'}
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </SidebarLayout>
  );
}
