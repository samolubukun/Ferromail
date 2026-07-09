'use client';

import React, { useState, useEffect, useRef } from 'react';
import {
  Send, Activity, CheckCircle2, AlertOctagon, Globe,
  Terminal, Mail, TrendingUp, TrendingDown, BarChart3,
  Zap, Loader2, RefreshCw,
} from 'lucide-react';
import SidebarLayout from '@/components/SidebarLayout';

const API = 'http://localhost:3000';

interface CampaignRow {
  id: string;
  name: string;
  status: string;
  subject: string;
  totalRecipients: number;
  sentCount: number;
  deliveredCount: number;
  openedCount: number;
  clickedCount: number;
  bouncedCount: number;
  sentAt: string | null;
  createdAt: string;
}

interface StatData {
  totalSent:       number;
  deliveryRate:    string;
  hardBounces:     number;
  activeDomains:   number;
  totalDomains:    number;
  totalContacts:   number;
}

interface ChartPoint { label: string; value: number; }

export default function Dashboard() {
  const [from, setFrom]       = useState('');
  const [to, setTo]           = useState('');
  const [subject, setSubject] = useState('');
  const [body, setBody]       = useState('');
  const [apiKey, setApiKey]   = useState('');

  const [isSending, setIsSending]   = useState(false);
  const [apiResponse, setApiResp]   = useState<any | null>(null);
  const [showApiKey, setShowApiKey] = useState(false);
  const [barsVisible, setBars]      = useState(false);

  const [stats, setStats]       = useState<StatData | null>(null);
  const [campaigns, setCampaigns] = useState<CampaignRow[]>([]);
  const [chartData, setChart]   = useState<ChartPoint[]>([]);
  const [isLoadingStats, setLoadingStats] = useState(true);

  const token = () => localStorage.getItem('ferromail_token') || '';

  const loadStats = async () => {
    setLoadingStats(true);
    setBars(false);
    try {
      const [campRes, contactRes, domainRes] = await Promise.all([
        fetch(`${API}/v1/campaigns`, { headers: { Authorization: `Bearer ${token()}` } }),
        fetch(`${API}/v1/contacts`,  { headers: { Authorization: `Bearer ${token()}` } }),
        fetch(`${API}/v1/domains`,   { headers: { Authorization: `Bearer ${token()}` } }),
      ]);

      const campData:    CampaignRow[]  = campRes.ok    ? await campRes.json()    : [];
      const contactData: any[]          = contactRes.ok ? await contactRes.json() : [];
      const domainData:  any[]          = domainRes.ok  ? await domainRes.json()  : [];

      setCampaigns(campData);

      // Aggregate from real campaign data
      const totalSent     = campData.reduce((s, c) => s + (c.sentCount || 0), 0);
      const totalDelivered = campData.reduce((s, c) => s + (c.deliveredCount || 0), 0);
      const hardBounces   = campData.reduce((s, c) => s + (c.bouncedCount || 0), 0);
      const deliveryRate  = totalSent > 0
        ? ((totalDelivered / totalSent) * 100).toFixed(2) + '%'
        : '—';
      const activeDomains = domainData.filter((d: any) => d.verified).length;

      setStats({
        totalSent,
        deliveryRate,
        hardBounces,
        activeDomains,
        totalDomains:  domainData.length,
        totalContacts: contactData.length,
      });

      // Build chart from campaigns grouped by day
      const dayMap: Record<string, number> = {};
      campData.forEach(c => {
        const date = (c.sentAt || c.createdAt).split('T')[0];
        dayMap[date] = (dayMap[date] || 0) + (c.sentCount || 0);
      });

      const sorted = Object.entries(dayMap)
        .sort(([a], [b]) => a.localeCompare(b))
        .slice(-7)
        .map(([date, value]) => ({
          label: new Date(date).toLocaleDateString('en-US', { weekday: 'short' }),
          value,
        }));

      setChart(sorted.length ? sorted : []);
    } catch (e) {
      console.error('Failed to load dashboard stats', e);
    } finally {
      setLoadingStats(false);
      setTimeout(() => setBars(true), 150);
    }
  };

  useEffect(() => {
    const t = localStorage.getItem('ferromail_token') || '';
    setApiKey(t);
    loadStats();
  }, []);

  const handleSend = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSending(true); setApiResp(null);
    try {
      const res  = await fetch(`${API}/v1/send`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${apiKey}` },
        body: JSON.stringify({ from, to: to.split(',').map(s => s.trim()), subject, body }),
      });
      const data = await res.json();
      setApiResp({ status: res.status, ok: res.ok, data });
    } catch (err: any) {
      setApiResp({ status: 'ERR', ok: false, data: { message: err.message || 'Network error' } });
    } finally {
      setIsSending(false);
    }
  };

  const maxChart = Math.max(...chartData.map(d => d.value), 1);

  const STATUS_COLOR: Record<string, string> = {
    SENT:      '#10b981',
    SENDING:   '#6366f1',
    SCHEDULED: '#38bdf8',
    DRAFT:     '#64748b',
    CANCELLED: '#ef4444',
  };

  return (
    <SidebarLayout>
      {/* ─── HEADER ─── */}
      <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', marginBottom: 28, gap: 16 }}>
        <div>
          <h2 style={{ fontFamily: 'Outfit', fontSize: 24, fontWeight: 800, color: '#f1f5f9', letterSpacing: '-0.02em', marginBottom: 4, display: 'flex', alignItems: 'center', gap: 8 }}>
            Platform Overview <Zap size={18} color="#6366f1" />
          </h2>
          <p style={{ fontSize: 13, color: 'var(--text-secondary)' }}>Live stats computed from your campaigns, contacts and domains.</p>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <button onClick={loadStats} className="btn-ghost" style={{ gap: 6 }}>
            <RefreshCw size={13} style={isLoadingStats ? { animation: 'spin 1s linear infinite' } : {}} />
            Refresh
          </button>
          {/* API Key bar */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, background: 'var(--bg-elevated)', border: '1px solid var(--border-default)', borderRadius: 10, padding: '7px 12px', minWidth: 240 }}>
            <Terminal size={12} color="var(--text-muted)" style={{ flexShrink: 0 }} />
            <input
              type={showApiKey ? 'text' : 'password'}
              value={apiKey}
              onChange={e => setApiKey(e.target.value)}
              placeholder="Bearer token…"
              style={{ background: 'transparent', border: 'none', outline: 'none', fontSize: 11, color: 'var(--text-secondary)', flex: 1, fontFamily: 'JetBrains Mono, monospace' }}
            />
            <button onClick={() => setShowApiKey(p => !p)} style={{ background: 'none', border: 'none', cursor: 'pointer', color: 'var(--text-muted)', fontSize: 10, fontWeight: 700, textTransform: 'uppercase' }}>
              {showApiKey ? 'Hide' : 'Show'}
            </button>
          </div>
        </div>
      </div>

      {/* ─── STAT CARDS ─── */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4,1fr)', gap: 14, marginBottom: 20 }}>
        {isLoadingStats ? (
          [1,2,3,4].map(i => (
            <div key={i} className="glass-panel" style={{ padding: '20px 22px' }}>
              <div className="skeleton" style={{ height: 12, width: '60%', marginBottom: 16, borderRadius: 6 }} />
              <div className="skeleton" style={{ height: 28, width: '80%', marginBottom: 12, borderRadius: 6 }} />
              <div className="skeleton" style={{ height: 10, width: '40%', borderRadius: 6 }} />
            </div>
          ))
        ) : stats ? [
          {
            label: 'Total Emails Sent',
            value: stats.totalSent.toLocaleString(),
            sub: `across ${campaigns.length} campaign${campaigns.length !== 1 ? 's' : ''}`,
            icon: Send, iconColor: '#818cf8', iconBg: 'rgba(99,102,241,0.1)',
            trend: null,
          },
          {
            label: 'Delivery Rate',
            value: stats.deliveryRate,
            sub: stats.totalSent > 0 ? 'delivered vs sent' : 'No emails sent yet',
            icon: CheckCircle2, iconColor: '#34d399', iconBg: 'rgba(16,185,129,0.1)',
            trend: stats.totalSent > 0 ? 'up' : null,
          },
          {
            label: 'Hard Bounces',
            value: stats.hardBounces.toLocaleString(),
            sub: stats.hardBounces === 0 ? 'Clean list!' : 'invalid addresses',
            icon: AlertOctagon, iconColor: '#fb7185', iconBg: 'rgba(239,68,68,0.1)',
            trend: stats.hardBounces > 0 ? 'down' : null,
          },
          {
            label: 'Verified Domains',
            value: `${stats.activeDomains} / ${stats.totalDomains}`,
            sub: stats.totalDomains === 0 ? 'No domains added' : stats.activeDomains === stats.totalDomains ? 'All verified' : `${stats.totalDomains - stats.activeDomains} pending DNS`,
            icon: Globe, iconColor: '#38bdf8', iconBg: 'rgba(56,189,248,0.1)',
            trend: null,
          },
        ].map(card => {
          const Icon = card.icon;
          return (
            <div key={card.label} className="glass-panel animate-fade-in-up" style={{ padding: '20px 22px' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 14 }}>
                <span style={{ fontSize: 12, fontWeight: 600, color: 'var(--text-secondary)' }}>{card.label}</span>
                <div style={{ width: 32, height: 32, borderRadius: 9, background: card.iconBg, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                  <Icon size={15} color={card.iconColor} />
                </div>
              </div>
              <div style={{ fontFamily: 'Outfit', fontSize: 26, fontWeight: 800, color: '#f1f5f9', letterSpacing: '-0.03em', lineHeight: 1, marginBottom: 10 }}>
                {card.value}
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 5, fontSize: 11, color: 'var(--text-muted)' }}>
                {card.trend === 'up'   && <TrendingUp  size={12} color="var(--success)" />}
                {card.trend === 'down' && <TrendingDown size={12} color="var(--danger)" />}
                {card.sub}
              </div>
            </div>
          );
        }) : null}
      </div>

      {/* ─── CHART ─── */}
      <div className="glass-panel" style={{ padding: '20px 24px', marginBottom: 20 }}>
        <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', marginBottom: 18 }}>
          <div>
            <h3 style={{ fontFamily: 'Outfit', fontSize: 15, fontWeight: 700, color: '#f1f5f9', marginBottom: 3, display: 'flex', alignItems: 'center', gap: 7 }}>
              <BarChart3 size={15} color="#6366f1" /> Sent Emails by Campaign Date
            </h3>
            <p style={{ fontSize: 12, color: 'var(--text-muted)' }}>Based on your real campaign data.</p>
          </div>
          <span style={{ fontSize: 11, color: 'var(--text-muted)', background: 'var(--bg-elevated)', border: '1px solid var(--border-subtle)', borderRadius: 7, padding: '4px 10px', fontFamily: 'JetBrains Mono, monospace' }}>
            Last {chartData.length} entries
          </span>
        </div>

        {isLoadingStats ? (
          <div style={{ height: 160, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
            <Loader2 size={24} color="#6366f1" style={{ animation: 'spin 1s linear infinite' }} />
          </div>
        ) : chartData.length === 0 ? (
          <div style={{ height: 140, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 8 }}>
            <BarChart3 size={28} color="var(--text-muted)" />
            <p style={{ fontSize: 13, color: 'var(--text-muted)' }}>No campaign data yet. Send your first campaign to see this chart.</p>
          </div>
        ) : (
          <div style={{ display: 'flex', alignItems: 'flex-end', gap: 12, height: 140 }}>
            {chartData.map((bar, idx) => {
              const pct = (bar.value / maxChart) * 100;
              return (
                <div key={idx} style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 6, height: '100%', justifyContent: 'flex-end' }}>
                  <span style={{ fontSize: 10, color: 'var(--text-muted)', fontFamily: 'JetBrains Mono, monospace', opacity: barsVisible ? 1 : 0, transition: 'opacity 0.3s' }}>
                    {bar.value.toLocaleString()}
                  </span>
                  <div style={{
                    width: '100%',
                    height: barsVisible ? `${Math.max(pct, 3)}%` : '0%',
                    background: 'linear-gradient(180deg, #818cf8 0%, #4f46e5 100%)',
                    borderRadius: '6px 6px 3px 3px',
                    transition: `height 0.6s cubic-bezier(0.4,0,0.2,1) ${idx * 70}ms`,
                    boxShadow: '0 0 10px rgba(99,102,241,0.2)',
                    minHeight: 3,
                  }} />
                  <span style={{ fontSize: 11, color: 'var(--text-muted)', fontWeight: 600 }}>{bar.label}</span>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* ─── BOTTOM DUAL PANEL ─── */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 20 }}>

        {/* Send Email Form */}
        <div className="glass-panel" style={{ padding: '20px 24px' }}>
          <div style={{ marginBottom: 18 }}>
            <h3 style={{ fontFamily: 'Outfit', fontSize: 15, fontWeight: 700, color: '#f1f5f9', display: 'flex', alignItems: 'center', gap: 7, marginBottom: 3 }}>
              <Send size={14} color="#6366f1" /> Send Transactional Email
            </h3>
            <p style={{ fontSize: 12, color: 'var(--text-muted)' }}>POST to the Rust Axum SMTP endpoint.</p>
          </div>

          <form onSubmit={handleSend} style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 10 }}>
              <div>
                <label className="label">From</label>
                <input type="email" value={from} onChange={e => setFrom(e.target.value)} required placeholder="you@domain.com" className="input-field" style={{ fontSize: 12 }} />
              </div>
              <div>
                <label className="label">To</label>
                <input type="text" value={to} onChange={e => setTo(e.target.value)} required placeholder="recipient@example.com" className="input-field" style={{ fontSize: 12 }} />
              </div>
            </div>
            <div>
              <label className="label">Subject</label>
              <input type="text" value={subject} onChange={e => setSubject(e.target.value)} required placeholder="Hello from Ferromail" className="input-field" style={{ fontSize: 12 }} />
            </div>
            <div>
              <label className="label">HTML Body</label>
              <textarea value={body} onChange={e => setBody(e.target.value)} required rows={4} placeholder="<p>Your message here.</p>" className="input-field" style={{ fontSize: 12, fontFamily: 'JetBrains Mono, monospace', resize: 'vertical' }} />
            </div>

            <button type="submit" disabled={isSending} className="btn-primary" style={{ width: '100%', justifyContent: 'center' }}>
              {isSending ? (
                <><span style={{ width: 14, height: 14, border: '2px solid rgba(255,255,255,0.3)', borderTopColor: 'white', borderRadius: '50%', animation: 'spin 0.8s linear infinite', display: 'inline-block' }} /> Sending…</>
              ) : (
                <><Send size={14} /> Send Email</>
              )}
            </button>
          </form>

          {apiResponse && (
            <div style={{ marginTop: 14, background: 'rgba(2,8,23,0.7)', border: `1px solid ${apiResponse.ok ? 'rgba(16,185,129,0.2)' : 'rgba(239,68,68,0.2)'}`, borderRadius: 10, padding: '12px 14px' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
                <span style={{ fontSize: 10, fontWeight: 700, letterSpacing: '0.08em', textTransform: 'uppercase', color: 'var(--text-muted)' }}>API Response</span>
                <span style={{ fontSize: 10, fontFamily: 'JetBrains Mono, monospace', fontWeight: 700, background: apiResponse.ok ? 'rgba(16,185,129,0.12)' : 'rgba(239,68,68,0.12)', color: apiResponse.ok ? '#34d399' : '#fb7185', padding: '2px 8px', borderRadius: 6 }}>
                  HTTP {apiResponse.status}
                </span>
              </div>
              <pre style={{ fontSize: 11, fontFamily: 'JetBrains Mono, monospace', color: '#94a3b8', whiteSpace: 'pre-wrap', wordBreak: 'break-all', maxHeight: 100, overflowY: 'auto' }}>
                {JSON.stringify(apiResponse.data, null, 2)}
              </pre>
            </div>
          )}
        </div>

        {/* Recent Campaigns */}
        <div className="glass-panel" style={{ padding: '20px 24px' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 18 }}>
            <div>
              <h3 style={{ fontFamily: 'Outfit', fontSize: 15, fontWeight: 700, color: '#f1f5f9', display: 'flex', alignItems: 'center', gap: 7, marginBottom: 3 }}>
                <Activity size={14} color="#10b981" /> Recent Campaigns
              </h3>
              <p style={{ fontSize: 12, color: 'var(--text-muted)' }}>Your latest broadcast activity.</p>
            </div>
          </div>

          {isLoadingStats ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
              {[1,2,3].map(i => <div key={i} className="skeleton" style={{ height: 56, borderRadius: 10 }} />)}
            </div>
          ) : campaigns.length === 0 ? (
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 10, padding: '32px 0', textAlign: 'center' }}>
              <Mail size={28} color="var(--text-muted)" />
              <p style={{ fontSize: 13, color: 'var(--text-muted)' }}>No campaigns yet.<br />Create one in the Campaigns tab.</p>
            </div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
              {campaigns.slice(0, 5).map(c => (
                <div key={c.id} style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '10px 12px', background: 'rgba(15,26,46,0.5)', border: '1px solid var(--border-subtle)', borderRadius: 10 }}>
                  <div style={{ width: 8, height: 8, borderRadius: '50%', background: STATUS_COLOR[c.status] || '#64748b', flexShrink: 0, boxShadow: `0 0 6px ${STATUS_COLOR[c.status] || '#64748b'}` }} />
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <p style={{ fontSize: 12, fontWeight: 600, color: '#e2e8f0', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{c.name}</p>
                    <p style={{ fontSize: 11, color: 'var(--text-muted)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{c.subject}</p>
                  </div>
                  <div style={{ textAlign: 'right', flexShrink: 0 }}>
                    <p style={{ fontSize: 11, fontFamily: 'JetBrains Mono, monospace', color: STATUS_COLOR[c.status] || '#64748b', fontWeight: 700, textTransform: 'uppercase' }}>{c.status}</p>
                    <p style={{ fontSize: 10, color: 'var(--text-muted)', fontFamily: 'JetBrains Mono, monospace' }}>
                      {c.sentCount > 0 ? `${c.sentCount.toLocaleString()} sent` : c.createdAt.split('T')[0]}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </SidebarLayout>
  );
}
