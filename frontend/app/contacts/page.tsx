'use client';

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Users, Plus, Trash2, Upload, Search, Loader2, CheckCircle2, FileSpreadsheet, X, AlertCircle } from 'lucide-react';
import SidebarLayout from '@/components/SidebarLayout';

interface Contact {
  id: string;
  email: string;
  subscribed: boolean;
  data: Record<string, any> | null;
  createdAt: string;
}

function Avatar({ email }: { email: string }) {
  const initials = email.slice(0, 2).toUpperCase();
  let hue = 0;
  for (let i = 0; i < email.length; i++) { hue += email.charCodeAt(i); }
  hue = hue % 360;
  return (
    <div style={{
      width: 32, height: 32, borderRadius: 9, flexShrink: 0,
      background: `hsl(${hue}, 50%, 20%)`,
      border: `1px solid hsl(${hue}, 50%, 30%)`,
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      fontSize: 11, fontWeight: 700, color: `hsl(${hue}, 70%, 70%)`,
      fontFamily: 'Outfit, sans-serif',
    }}>
      {initials}
    </div>
  );
}

export default function Contacts() {
  const [contacts, setContacts]           = useState<Contact[]>([]);
  const [filtered, setFiltered]           = useState<Contact[]>([]);
  const [search, setSearch]               = useState('');
  const [email, setEmail]                 = useState('');
  const [firstName, setFirstName]         = useState('');
  const [lastName, setLastName]           = useState('');
  const [isLoading, setIsLoading]         = useState(true);
  const [isAdding, setIsAdding]           = useState(false);
  const [isImporting, setIsImporting]     = useState(false);
  const [importStatus, setImportStatus]   = useState<string | null>(null);
  const [error, setError]                 = useState<string | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);
  const [isDragOver, setIsDragOver]       = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);

  const token = () => localStorage.getItem('ferromail_token') || '';

  const fetchContacts = async () => {
    setIsLoading(true); setError(null);
    try {
      const res  = await fetch('http://localhost:3000/v1/contacts', { headers: { Authorization: `Bearer ${token()}` } });
      const data = await res.json();
      if (res.ok) { setContacts(data); setFiltered(data); }
      else setError(data.error || 'Failed to load contacts.');
    } catch { setError('Cannot connect to backend.'); }
    finally { setIsLoading(false); }
  };

  useEffect(() => { fetchContacts(); }, []);

  useEffect(() => {
    const q = search.toLowerCase().trim();
    setFiltered(!q ? contacts : contacts.filter(c =>
      c.email.toLowerCase().includes(q) || JSON.stringify(c.data || {}).toLowerCase().includes(q)
    ));
  }, [search, contacts]);

  const handleAdd = async (e: React.FormEvent) => {
    e.preventDefault(); setIsAdding(true); setError(null);
    const customFields = firstName || lastName ? { firstName, lastName } : null;
    try {
      const res  = await fetch('http://localhost:3000/v1/contacts', { method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token()}` }, body: JSON.stringify({ email, data: customFields }) });
      const data = await res.json();
      if (res.ok) { setEmail(''); setFirstName(''); setLastName(''); fetchContacts(); }
      else setError(data.error || 'Failed to add contact.');
    } catch { setError('Connection error.'); }
    finally { setIsAdding(false); }
  };

  const handleDelete = async (id: string) => {
    setDeleteConfirm(null);
    try {
      const res = await fetch(`http://localhost:3000/v1/contacts/${id}`, { method: 'DELETE', headers: { Authorization: `Bearer ${token()}` } });
      if (res.ok) fetchContacts();
      else { const d = await res.json(); setError(d.error || 'Delete failed.'); }
    } catch { setError('Connection error.'); }
  };

  const processCsv = useCallback(async (file: File) => {
    setIsImporting(true); setImportStatus('Parsing CSV…'); setError(null);
    const reader = new FileReader();
    reader.onload = async (ev) => {
      try {
        const text  = ev.target?.result as string;
        const lines = text.split('\n');
        if (lines.length < 2) { setError('CSV appears to be empty.'); setIsImporting(false); return; }
        const headers = lines[0].toLowerCase().split(',').map(h => h.trim());
        const eIdx = headers.indexOf('email') !== -1 ? headers.indexOf('email') : 0;
        const fIdx = ['firstname', 'first_name', 'name'].map(k => headers.indexOf(k)).find(i => i !== -1) ?? -1;
        const lIdx = ['lastname', 'last_name'].map(k => headers.indexOf(k)).find(i => i !== -1) ?? -1;

        const parsed = lines.slice(1).reduce<any[]>((acc, line) => {
          const row = line.split(',').map(r => r.trim());
          if (!row[eIdx]?.includes('@')) return acc;
          const d: Record<string, string> = {};
          if (fIdx !== -1 && row[fIdx]) d.firstName = row[fIdx];
          if (lIdx !== -1 && row[lIdx]) d.lastName  = row[lIdx];
          acc.push({ email: row[eIdx], data: Object.keys(d).length ? d : null });
          return acc;
        }, []);

        if (!parsed.length) { setError("No valid email rows found. Ensure there's an 'email' column."); setIsImporting(false); return; }
        setImportStatus(`Importing ${parsed.length} contacts…`);

        const res  = await fetch('http://localhost:3000/v1/contacts/csv', { method: 'POST', headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token()}` }, body: JSON.stringify({ contacts: parsed }) });
        const data = await res.json();
        if (res.ok && data.success) {
          setImportStatus(`Successfully imported ${data.imported} contacts.`);
          fetchContacts();
          setTimeout(() => setImportStatus(null), 4000);
        } else setError(data.error || 'CSV import failed.');
      } catch (err: any) { setError(err.message || 'Error processing file.'); }
      finally { setIsImporting(false); }
    };
    reader.readAsText(file);
  }, []);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const f = e.target.files?.[0];
    if (f) processCsv(f);
    e.target.value = '';
  };

  const subscribed   = contacts.filter(c => c.subscribed).length;
  const unsubscribed = contacts.length - subscribed;

  return (
    <SidebarLayout>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', marginBottom: 28 }}>
        <div>
          <h2 style={{ fontFamily: 'Outfit', fontSize: 24, fontWeight: 800, color: '#f1f5f9', letterSpacing: '-0.02em', marginBottom: 4 }}>Audience</h2>
          <p style={{ fontSize: 13, color: 'var(--text-secondary)' }}>Manage contacts, segments, and CSV imports.</p>
        </div>
        <div style={{ display: 'flex', gap: 10 }}>
          <input type="file" ref={fileRef} onChange={handleFileChange} accept=".csv" style={{ display: 'none' }} />
          <button onClick={() => fileRef.current?.click()} disabled={isImporting} className="btn-secondary" style={{ gap: 8 }}>
            {isImporting ? <Loader2 size={13} style={{ animation: 'spin 1s linear infinite' }} /> : <Upload size={13} color="#818cf8" />}
            Import CSV
          </button>
        </div>
      </div>

      {/* Stat cards */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3,1fr)', gap: 14, marginBottom: 24 }}>
        {[
          { label: 'Total contacts', value: contacts.length, color: '#818cf8', bg: 'rgba(99,102,241,0.08)' },
          { label: 'Subscribed',     value: subscribed,      color: '#34d399', bg: 'rgba(16,185,129,0.08)' },
          { label: 'Unsubscribed',   value: unsubscribed,    color: '#fb7185', bg: 'rgba(239,68,68,0.08)' },
        ].map(s => (
          <div key={s.label} style={{ background: s.bg, border: `1px solid ${s.color}22`, borderRadius: 12, padding: '16px 20px' }}>
            <div style={{ fontSize: 12, color: 'var(--text-muted)', marginBottom: 6, fontWeight: 500 }}>{s.label}</div>
            <div style={{ fontFamily: 'Outfit', fontSize: 28, fontWeight: 800, color: s.color }}>{s.value.toLocaleString()}</div>
          </div>
        ))}
      </div>

      {/* Feedback */}
      {importStatus && <div className="alert-success" style={{ marginBottom: 16 }}><CheckCircle2 size={14} />{importStatus}</div>}
      {error        && <div className="alert-error"   style={{ marginBottom: 16 }}><AlertCircle size={14} />{error}<button onClick={() => setError(null)} style={{ marginLeft: 'auto', background: 'none', border: 'none', cursor: 'pointer', color: 'inherit' }}><X size={14} /></button></div>}

      {/* Main grid */}
      <div style={{ display: 'grid', gridTemplateColumns: '280px 1fr', gap: 20, alignItems: 'start' }}>

        {/* Add Contact Form */}
        <div className="glass-panel" style={{ padding: '20px 22px', position: 'sticky', top: 24 }}>
          <h3 style={{ fontFamily: 'Outfit', fontSize: 14, fontWeight: 700, color: '#f1f5f9', marginBottom: 16, display: 'flex', alignItems: 'center', gap: 7 }}>
            <Plus size={14} color="#818cf8" /> Add Contact
          </h3>
          <form onSubmit={handleAdd} style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            <div>
              <label className="label">Email *</label>
              <input type="email" value={email} onChange={e => setEmail(e.target.value)} placeholder="contact@domain.com" required className="input-field" />
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
              <div>
                <label className="label">First name</label>
                <input type="text" value={firstName} onChange={e => setFirstName(e.target.value)} placeholder="John" className="input-field" />
              </div>
              <div>
                <label className="label">Last name</label>
                <input type="text" value={lastName} onChange={e => setLastName(e.target.value)} placeholder="Doe" className="input-field" />
              </div>
            </div>
            <button type="submit" disabled={isAdding} className="btn-primary" style={{ width: '100%', justifyContent: 'center', marginTop: 4 }}>
              {isAdding
                ? <><span style={{ width: 14, height: 14, border: '2px solid rgba(255,255,255,0.3)', borderTopColor: 'white', borderRadius: '50%', animation: 'spin 0.8s linear infinite', display: 'inline-block' }} /> Adding…</>
                : <><Plus size={13} /> Save Contact</>
              }
            </button>
          </form>

          {/* CSV drag zone */}
          <div
            onDragOver={e => { e.preventDefault(); setIsDragOver(true); }}
            onDragLeave={() => setIsDragOver(false)}
            onDrop={e => { e.preventDefault(); setIsDragOver(false); const f = e.dataTransfer.files[0]; if (f?.name.endsWith('.csv')) processCsv(f); }}
            onClick={() => fileRef.current?.click()}
            style={{
              marginTop: 16,
              border: `1.5px dashed ${isDragOver ? '#6366f1' : 'var(--border-default)'}`,
              borderRadius: 10,
              padding: '18px 12px',
              textAlign: 'center',
              cursor: 'pointer',
              background: isDragOver ? 'rgba(99,102,241,0.06)' : 'transparent',
              transition: 'all 0.2s',
            }}
          >
            <FileSpreadsheet size={20} color={isDragOver ? '#818cf8' : 'var(--text-muted)'} style={{ margin: '0 auto 6px' }} />
            <p style={{ fontSize: 11, color: 'var(--text-muted)', lineHeight: 1.5 }}>
              Drop a <strong style={{ color: '#818cf8' }}>.csv</strong> file here<br />or click to browse
            </p>
          </div>
        </div>

        {/* Contacts table */}
        <div>
          {/* Search bar */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, background: 'var(--bg-elevated)', border: '1px solid var(--border-default)', borderRadius: 10, padding: '9px 14px', marginBottom: 14, transition: 'border-color 0.2s' }}
            onFocusCapture={e => (e.currentTarget.style.borderColor = '#6366f1')}
            onBlurCapture={e => (e.currentTarget.style.borderColor = 'var(--border-default)')}
          >
            <Search size={14} color="var(--text-muted)" style={{ flexShrink: 0 }} />
            <input
              type="text"
              value={search}
              onChange={e => setSearch(e.target.value)}
              placeholder="Search by email or name…"
              style={{ background: 'transparent', border: 'none', outline: 'none', fontSize: 13, color: 'var(--text-primary)', flex: 1 }}
            />
            {search && <button onClick={() => setSearch('')} style={{ background: 'none', border: 'none', cursor: 'pointer', color: 'var(--text-muted)', display: 'flex', alignItems: 'center' }}><X size={13} /></button>}
          </div>

          {isLoading ? (
            <div className="glass-panel" style={{ padding: 60, display: 'flex', justifyContent: 'center' }}>
              <Loader2 size={28} color="#6366f1" style={{ animation: 'spin 1s linear infinite' }} />
            </div>
          ) : filtered.length === 0 ? (
            <div className="glass-panel">
              <div className="empty-state">
                <div className="empty-state-icon"><Users size={22} /></div>
                <h3 style={{ fontFamily: 'Outfit', fontSize: 15, fontWeight: 700, color: '#f1f5f9' }}>
                  {search ? 'No results found' : 'No contacts yet'}
                </h3>
                <p style={{ fontSize: 13, color: 'var(--text-secondary)', maxWidth: 280 }}>
                  {search ? `No contacts match "${search}".` : 'Add contacts manually or import a CSV file.'}
                </p>
              </div>
            </div>
          ) : (
            <div className="glass-panel" style={{ overflow: 'hidden' }}>
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Contact</th>
                    <th>Custom fields</th>
                    <th>Status</th>
                    <th>Joined</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {filtered.map(c => (
                    <tr key={c.id}>
                      <td>
                        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                          <Avatar email={c.email} />
                          <span style={{ fontWeight: 600, color: '#f1f5f9', fontSize: 13 }}>{c.email}</span>
                        </div>
                      </td>
                      <td>
                        {c.data ? (
                          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
                            {Object.entries(c.data).map(([k, v]) => (
                              <span key={k} style={{ fontSize: 10, background: 'var(--bg-elevated)', border: '1px solid var(--border-subtle)', borderRadius: 5, padding: '2px 7px', color: 'var(--text-secondary)', fontFamily: 'JetBrains Mono, monospace' }}>
                                {k}: {String(v)}
                              </span>
                            ))}
                          </div>
                        ) : <span style={{ color: 'var(--text-muted)', fontSize: 12 }}>—</span>}
                      </td>
                      <td>
                        {c.subscribed
                          ? <span className="badge badge-success">Subscribed</span>
                          : <span className="badge badge-danger">Unsubscribed</span>}
                      </td>
                      <td style={{ fontFamily: 'JetBrains Mono, monospace', fontSize: 11, color: 'var(--text-muted)' }}>
                        {c.createdAt.split('T')[0]}
                      </td>
                      <td style={{ textAlign: 'right' }}>
                        {deleteConfirm === c.id ? (
                          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                            <span style={{ fontSize: 11, color: 'var(--text-muted)' }}>Sure?</span>
                            <button onClick={() => handleDelete(c.id)} className="btn-danger" style={{ padding: '3px 9px', fontSize: 11 }}>Delete</button>
                            <button onClick={() => setDeleteConfirm(null)} className="btn-ghost" style={{ padding: '3px 9px', fontSize: 11 }}>Cancel</button>
                          </div>
                        ) : (
                          <button onClick={() => setDeleteConfirm(c.id)} className="btn-ghost" style={{ color: 'var(--text-muted)' }}>
                            <Trash2 size={13} />
                          </button>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              <div style={{ padding: '10px 16px', borderTop: '1px solid var(--border-subtle)', fontSize: 11, color: 'var(--text-muted)' }}>
                Showing {filtered.length} of {contacts.length} contacts
              </div>
            </div>
          )}
        </div>
      </div>
    </SidebarLayout>
  );
}
