'use client';

import React, { useState } from 'react';
import { useRouter } from 'next/navigation';
import { Cpu, Lock, Mail, ArrowRight, Eye, EyeOff, AlertCircle, UserPlus, Layers, Shield, Activity } from 'lucide-react';
import Link from 'next/link';
import Logo from '@/components/Logo';

const features = [
  { icon: Layers,   title: 'No per-email fees',          desc: 'Self-hosted infrastructure means zero markup on your volume.' },
  { icon: Shield,   title: 'Full SMTP relay with TLS',   desc: 'Secure outbound transmission matching standard mail client specs.' },
  { icon: Mail,     title: 'Campaigns & automation',     desc: 'Design newsletters and visual workflows to automate sequences.' },
  { icon: Activity, title: 'Real-time delivery tracking', desc: 'Instantly record delivery events, bounces, and complaints.' },
];

function getStrength(pw: string): { score: number; label: string; color: string } {
  let score = 0;
  if (pw.length >= 8)  score++;
  if (/[A-Z]/.test(pw)) score++;
  if (/[0-9]/.test(pw)) score++;
  if (/[^A-Za-z0-9]/.test(pw)) score++;
  const labels = ['', 'Weak', 'Fair', 'Good', 'Strong'];
  const colors = ['', '#ef4444', '#f59e0b', '#10b981', '#6366f1'];
  return { score, label: labels[score] || '', color: colors[score] || '' };
}

export default function Signup() {
  const [email, setEmail]               = useState('');
  const [password, setPassword]         = useState('');
  const [confirmPassword, setConfirm]   = useState('');
  const [showPass, setShowPass]         = useState(false);
  const [showConfirm, setShowConfirm]   = useState(false);
  const [error, setError]               = useState<string | null>(null);
  const [isLoading, setIsLoading]       = useState(false);
  const router = useRouter();

  const strength = getStrength(password);
  const passwordsMatch = confirmPassword.length > 0 && password === confirmPassword;
  const passwordsMismatch = confirmPassword.length > 0 && password !== confirmPassword;

  const handleSignup = async (e: React.FormEvent) => {
    e.preventDefault();
    if (password !== confirmPassword) { setError('Passwords do not match.'); return; }
    setIsLoading(true);
    setError(null);

    try {
      const response = await fetch('http://localhost:3000/v1/auth/signup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password }),
      });

      const data = await response.json();

      if (response.ok && data.success) {
        localStorage.setItem('ferromail_token', data.token);
        localStorage.setItem('ferromail_project_id', data.project_id || '');
        localStorage.setItem('ferromail_email', email);
        router.push('/');
      } else {
        setError(data.error || 'Registration failed. Please check your input.');
      }
    } catch (err: any) {
      setError('Unable to connect to the Ferromail API.');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div style={{ minHeight: '100vh', display: 'flex', background: 'var(--bg-base)' }}>
      {/* LEFT: Decorative panel */}
      <div style={{
        width: '45%',
        background: 'linear-gradient(145deg, #0d1526 0%, #0a0f1e 50%, #06080f 100%)',
        borderRight: '1px solid var(--border-subtle)',
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'space-between',
        padding: '48px 52px',
        position: 'relative',
        overflow: 'hidden',
      }}>
        {/* Background glow blobs */}
        <div style={{
          position: 'absolute', top: '20%', left: '10%',
          width: 300, height: 300,
          borderRadius: '50%',
          background: 'radial-gradient(circle, rgba(99,102,241,0.15) 0%, transparent 70%)',
          filter: 'blur(40px)',
          pointerEvents: 'none',
        }} />
        <div style={{
          position: 'absolute', bottom: '15%', right: '5%',
          width: 200, height: 200,
          borderRadius: '50%',
          background: 'radial-gradient(circle, rgba(139,92,246,0.12) 0%, transparent 70%)',
          filter: 'blur(30px)',
          pointerEvents: 'none',
        }} />

        {/* Logo */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, position: 'relative' }}>
          <div style={{
            boxShadow: '0 0 28px rgba(99,102,241,0.25)',
            borderRadius: 13,
            overflow: 'hidden'
          }}>
            <Logo size={42} />
          </div>
          <div>
            <div style={{ fontFamily: 'Outfit', fontWeight: 800, fontSize: 20, color: '#f1f5f9', letterSpacing: '-0.02em' }}>Ferromail</div>
            <div style={{ fontSize: 10, fontWeight: 700, letterSpacing: '0.1em', textTransform: 'uppercase', color: '#6366f1', fontFamily: 'JetBrains Mono, monospace' }}>Email Infrastructure</div>
          </div>
        </div>

        {/* Headline */}
        <div style={{ position: 'relative' }}>
          <h1 style={{
            fontFamily: 'Outfit',
            fontSize: 36,
            fontWeight: 900,
            color: '#f1f5f9',
            letterSpacing: '-0.03em',
            lineHeight: 1.15,
            marginBottom: 16,
          }}>
            Start sending<br />
            <span style={{ background: 'linear-gradient(135deg, #818cf8, #6366f1)', WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent' }}>
              in minutes
            </span>
          </h1>
          <p style={{ fontSize: 14, color: 'var(--text-secondary)', lineHeight: 1.7, marginBottom: 40 }}>
            Create your workspace and start sending transactional and marketing emails through a self-hosted Rust backend.
          </p>

          <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
            {features.map(({ icon: Icon, title, desc }) => (
              <div key={title} style={{ display: 'flex', alignItems: 'flex-start', gap: 14 }}>
                <div style={{
                  width: 36, height: 36,
                  borderRadius: 10,
                  background: 'rgba(99,102,241,0.1)',
                  border: '1px solid rgba(99,102,241,0.2)',
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  flexShrink: 0,
                }}>
                  <Icon size={16} color="#818cf8" />
                </div>
                <div>
                  <div style={{ fontSize: 13, fontWeight: 600, color: '#e2e8f0', marginBottom: 2 }}>{title}</div>
                  <div style={{ fontSize: 12, color: 'var(--text-muted)', lineHeight: 1.5 }}>{desc}</div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Footer */}
        <div style={{ fontSize: 11, color: 'var(--text-muted)', fontFamily: 'JetBrains Mono, monospace' }}>
          © 2026 Ferromail. Open source email platform.
        </div>
      </div>

      {/* RIGHT: Signup form */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', padding: '48px 40px' }}>
        <div style={{ width: '100%', maxWidth: 400 }}>
          <div style={{ marginBottom: 32 }}>
            <h2 style={{ fontFamily: 'Outfit', fontSize: 26, fontWeight: 800, color: '#f1f5f9', letterSpacing: '-0.02em', marginBottom: 8 }}>
              Create your workspace
            </h2>
            <p style={{ fontSize: 14, color: 'var(--text-secondary)' }}>
              Your own high-performance email backend
            </p>
          </div>

          {error && (
            <div className="alert-error" style={{ marginBottom: 20 }}>
              <AlertCircle size={15} style={{ flexShrink: 0 }} />
              {error}
            </div>
          )}

          <form onSubmit={handleSignup} style={{ display: 'flex', flexDirection: 'column', gap: 18 }}>
            <div>
              <label className="label">Email address</label>
              <div style={{ position: 'relative' }}>
                <Mail size={15} style={{ position: 'absolute', left: 14, top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)', pointerEvents: 'none' }} />
                <input type="email" required value={email} onChange={e => setEmail(e.target.value)} placeholder="you@company.com" className="input-field" style={{ paddingLeft: 40 }} />
              </div>
            </div>

            <div>
              <label className="label">Password</label>
              <div style={{ position: 'relative' }}>
                <Lock size={15} style={{ position: 'absolute', left: 14, top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)', pointerEvents: 'none' }} />
                <input type={showPass ? 'text' : 'password'} required value={password} onChange={e => setPassword(e.target.value)} placeholder="Min 8 characters" className="input-field" style={{ paddingLeft: 40, paddingRight: 44 }} />
                <button type="button" onClick={() => setShowPass(p => !p)} style={{ position: 'absolute', right: 12, top: '50%', transform: 'translateY(-50%)', background: 'none', border: 'none', cursor: 'pointer', color: 'var(--text-muted)', padding: 4, display: 'flex', alignItems: 'center' }}>
                  {showPass ? <EyeOff size={15} /> : <Eye size={15} />}
                </button>
              </div>

              {/* Password strength bar */}
              {password.length > 0 && (
                <div style={{ marginTop: 8 }}>
                  <div style={{ display: 'flex', gap: 4, marginBottom: 4 }}>
                    {[1, 2, 3, 4].map(i => (
                      <div key={i} style={{
                        flex: 1, height: 3, borderRadius: 99,
                        background: i <= strength.score ? strength.color : 'var(--border-default)',
                        transition: 'background 0.3s',
                      }} />
                    ))}
                  </div>
                  <div style={{ fontSize: 11, color: strength.color, fontWeight: 600 }}>{strength.label}</div>
                </div>
              )}
            </div>

            <div>
              <label className="label">Confirm password</label>
              <div style={{ position: 'relative' }}>
                <Lock size={15} style={{ position: 'absolute', left: 14, top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)', pointerEvents: 'none' }} />
                <input
                  type={showConfirm ? 'text' : 'password'}
                  required
                  value={confirmPassword}
                  onChange={e => setConfirm(e.target.value)}
                  placeholder="Repeat password"
                  className="input-field"
                  style={{
                    paddingLeft: 40, paddingRight: 44,
                    borderColor: passwordsMismatch ? 'var(--danger)' : passwordsMatch ? 'var(--success)' : undefined,
                  }}
                />
                <button type="button" onClick={() => setShowConfirm(p => !p)} style={{ position: 'absolute', right: 12, top: '50%', transform: 'translateY(-50%)', background: 'none', border: 'none', cursor: 'pointer', color: 'var(--text-muted)', padding: 4, display: 'flex', alignItems: 'center' }}>
                  {showConfirm ? <EyeOff size={15} /> : <Eye size={15} />}
                </button>
              </div>
              {passwordsMismatch && <p style={{ fontSize: 11, color: 'var(--danger)', marginTop: 5 }}>Passwords don't match</p>}
              {passwordsMatch && <p style={{ fontSize: 11, color: 'var(--success)', marginTop: 5 }}>Passwords match</p>}
            </div>

            <button type="submit" disabled={isLoading} className="btn-primary" style={{ marginTop: 4, width: '100%', padding: '12px 20px', fontSize: 15 }}>
              {isLoading ? (
                <>
                  <span style={{ width: 16, height: 16, border: '2px solid rgba(255,255,255,0.3)', borderTopColor: 'white', borderRadius: '50%', animation: 'spin 0.8s linear infinite', display: 'inline-block' }} />
                  Creating workspace...
                </>
              ) : (
                <>
                  <UserPlus size={15} /> Create workspace
                </>
              )}
            </button>
          </form>

          <div style={{ marginTop: 28, paddingTop: 28, borderTop: '1px solid var(--border-subtle)', textAlign: 'center' }}>
            <p style={{ fontSize: 13, color: 'var(--text-muted)' }}>
              Already have an account?{' '}
              <Link href="/login" style={{ color: '#818cf8', fontWeight: 600, textDecoration: 'none' }}>Sign in</Link>
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
