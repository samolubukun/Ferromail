'use client';

import React, { useEffect, useState } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import {
  LayoutDashboard,
  Mail,
  Globe,
  Settings,
  Loader2,
  LogOut,
  Users,
  Zap,
  ChevronRight,
  Cpu,
} from 'lucide-react';
import Link from 'next/link';
import Logo from './Logo';

const navLinks = [
  { name: 'Dashboard',  href: '/',           icon: LayoutDashboard },
  { name: 'Campaigns',  href: '/campaigns',  icon: Mail },
  { name: 'Audience',   href: '/contacts',   icon: Users },
  { name: 'Workflows',  href: '/workflows',  icon: Zap },
  { name: 'Domains',    href: '/domains',    icon: Globe },
  { name: 'Settings',   href: '/settings',  icon: Settings },
];

export default function SidebarLayout({ children }: { children: React.ReactNode }) {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [userEmail, setUserEmail] = useState('');
  const router = useRouter();
  const pathname = usePathname();

  useEffect(() => {
    const token = localStorage.getItem('ferromail_token');
    let email   = localStorage.getItem('ferromail_email') || '';

    // Fallback: decode email from JWT payload if not stored
    if (!email && token) {
      try {
        const payload = JSON.parse(atob(token.split('.')[1]));
        email = payload.email || payload.sub || '';
        if (email) localStorage.setItem('ferromail_email', email);
      } catch {}
    }

    if (!token) {
      router.push('/login');
    } else {
      setIsAuthenticated(true);
      setUserEmail(email);
    }
  }, [router]);

  const handleLogout = () => {
    localStorage.removeItem('ferromail_token');
    localStorage.removeItem('ferromail_project_id');
    localStorage.removeItem('ferromail_email');
    router.push('/login');
  };

  if (!isAuthenticated) {
    return (
      <div style={{ minHeight: '100vh', background: 'var(--bg-base)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '16px' }}>
          <div style={{
            boxShadow: '0 0 24px rgba(99,102,241,0.25)',
            borderRadius: 14,
            overflow: 'hidden'
          }}>
            <Logo size={48} />
          </div>
          <Loader2 size={20} color="#6366f1" style={{ animation: 'spin 1s linear infinite' }} />
        </div>
      </div>
    );
  }

  const initials = userEmail
    ? userEmail.slice(0, 2).toUpperCase()
    : 'FM';

  return (
    <div style={{
      minHeight: '100vh',
      display: 'flex',
      background: 'var(--bg-base)',
      position: 'relative',
      zIndex: 1,
    }}>
      {/* ===================== SIDEBAR ===================== */}
      <aside style={{
        width: 240,
        minHeight: '100vh',
        display: 'flex',
        flexDirection: 'column',
        background: 'rgba(10,17,32,0.9)',
        backdropFilter: 'blur(20px)',
        borderRight: '1px solid var(--border-subtle)',
        position: 'sticky',
        top: 0,
        flexShrink: 0,
        zIndex: 20,
      }}>
        {/* Brand */}
        <div style={{ padding: '24px 20px 20px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <div style={{
              boxShadow: '0 0 20px rgba(99,102,241,0.2)',
              borderRadius: 11,
              overflow: 'hidden',
              flexShrink: 0,
            }}>
              <Logo size={38} />
            </div>
            <div>
              <div style={{
                fontFamily: 'Outfit, sans-serif',
                fontWeight: 800,
                fontSize: 17,
                color: '#f1f5f9',
                letterSpacing: '-0.02em',
                lineHeight: 1.1,
              }}>
                Ferromail
              </div>
              <div style={{
                fontSize: 9,
                fontWeight: 700,
                letterSpacing: '0.12em',
                textTransform: 'uppercase',
                color: '#6366f1',
                fontFamily: 'JetBrains Mono, monospace',
                marginTop: 2,
              }}>
                Rust Engine
              </div>
            </div>
          </div>
        </div>

        {/* Nav Divider */}
        <div style={{ height: 1, background: 'var(--border-subtle)', margin: '0 16px 12px' }} />

        {/* Nav Links */}
        <nav style={{ padding: '0 10px', flex: 1 }}>
          <div style={{ fontSize: 10, fontWeight: 700, letterSpacing: '0.1em', textTransform: 'uppercase', color: 'var(--text-muted)', padding: '4px 10px 10px' }}>
            Navigation
          </div>
          {navLinks.map((link) => {
            const Icon = link.icon;
            const isActive = pathname === link.href || (link.href !== '/' && pathname.startsWith(link.href));
            return (
              <Link
                key={link.name}
                href={link.href}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 10,
                  padding: '9px 12px',
                  borderRadius: 10,
                  marginBottom: 2,
                  fontSize: 13,
                  fontWeight: isActive ? 600 : 500,
                  color: isActive ? '#a5b4fc' : 'var(--text-secondary)',
                  background: isActive ? 'rgba(99,102,241,0.1)' : 'transparent',
                  border: `1px solid ${isActive ? 'rgba(99,102,241,0.2)' : 'transparent'}`,
                  textDecoration: 'none',
                  transition: 'all 0.15s ease',
                  position: 'relative',
                }}
                onMouseEnter={(e) => {
                  if (!isActive) {
                    e.currentTarget.style.background = 'rgba(148,163,184,0.05)';
                    e.currentTarget.style.color = '#cbd5e1';
                  }
                }}
                onMouseLeave={(e) => {
                  if (!isActive) {
                    e.currentTarget.style.background = 'transparent';
                    e.currentTarget.style.color = 'var(--text-secondary)';
                  }
                }}
              >
                {/* Active indicator bar */}
                {isActive && (
                  <div style={{
                    position: 'absolute',
                    left: 0, top: '50%', transform: 'translateY(-50%)',
                    width: 3, height: 18,
                    background: '#6366f1',
                    borderRadius: '0 3px 3px 0',
                  }} />
                )}
                <Icon size={15} style={{ flexShrink: 0, marginLeft: isActive ? 4 : 0 }} />
                <span style={{ flex: 1 }}>{link.name}</span>
                {isActive && <ChevronRight size={12} style={{ opacity: 0.5 }} />}
              </Link>
            );
          })}
        </nav>

        {/* Sidebar Footer */}
        <div style={{ padding: '12px 10px 20px', borderTop: '1px solid var(--border-subtle)', marginTop: 'auto' }}>
          {/* User row */}
          <div style={{
            display: 'flex',
            alignItems: 'center',
            gap: 10,
            padding: '8px 10px',
            borderRadius: 10,
            marginBottom: 6,
          }}>
            <div style={{
              width: 30, height: 30,
              borderRadius: 9,
              background: 'linear-gradient(135deg, rgba(99,102,241,0.3), rgba(79,70,229,0.3))',
              border: '1px solid rgba(99,102,241,0.3)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: 11,
              fontWeight: 700,
              color: '#a5b4fc',
              flexShrink: 0,
              fontFamily: 'Outfit, sans-serif',
            }}>
              {initials}
            </div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 12, fontWeight: 600, color: 'var(--text-primary)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                {userEmail || 'Workspace'}
              </div>
              <div style={{ fontSize: 10, color: 'var(--text-muted)', fontFamily: 'JetBrains Mono, monospace' }}>Active session</div>
            </div>
          </div>

          {/* Logout */}
          <button
            onClick={handleLogout}
            style={{
              width: '100%',
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              padding: '8px 12px',
              borderRadius: 9,
              fontSize: 13,
              fontWeight: 500,
              color: '#fb7185',
              background: 'transparent',
              border: 'none',
              cursor: 'pointer',
              transition: 'background 0.15s',
            }}
            onMouseEnter={e => { e.currentTarget.style.background = 'rgba(239,68,68,0.08)'; }}
            onMouseLeave={e => { e.currentTarget.style.background = 'transparent'; }}
          >
            <LogOut size={14} />
            Sign out
          </button>
        </div>
      </aside>

      {/* ===================== MAIN CONTENT ===================== */}
      <main style={{
        flex: 1,
        minWidth: 0,
        overflowY: 'auto',
        display: 'flex',
        flexDirection: 'column',
      }}>
        {/* Top bar */}
        <div style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '0 40px',
          height: 56,
          borderBottom: '1px solid var(--border-subtle)',
          background: 'rgba(10,17,32,0.7)',
          backdropFilter: 'blur(12px)',
          position: 'sticky',
          top: 0,
          zIndex: 10,
          flexShrink: 0,
        }}>
          {/* Current page breadcrumb */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 13 }}>
            <span style={{ color: 'var(--text-muted)' }}>Ferromail</span>
            <span style={{ color: 'var(--border-default)' }}>/</span>
            <span style={{ color: '#e2e8f0', fontWeight: 600 }}>
              {navLinks.find(l => l.href === pathname || (l.href !== '/' && pathname.startsWith(l.href)))?.name ?? 'Dashboard'}
            </span>
          </div>

          {/* User pill */}
          <div style={{
            display: 'flex',
            alignItems: 'center',
            gap: 10,
            background: 'var(--bg-elevated)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 99,
            padding: '5px 14px 5px 6px',
          }}>
            {/* Avatar */}
            <div style={{
              width: 26, height: 26,
              borderRadius: '50%',
              background: 'linear-gradient(135deg, rgba(99,102,241,0.4), rgba(79,70,229,0.4))',
              border: '1px solid rgba(99,102,241,0.35)',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              fontSize: 10, fontWeight: 700, color: '#a5b4fc',
              fontFamily: 'Outfit, sans-serif',
              flexShrink: 0,
            }}>
              {initials}
            </div>
            <span style={{
              fontSize: 12,
              fontWeight: 500,
              color: 'var(--text-secondary)',
              maxWidth: 180,
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}>
              {userEmail || 'Workspace'}
            </span>
          </div>
        </div>

        {/* Page content */}
        <div style={{
          flex: 1,
          padding: '32px 40px',
          maxWidth: 1200,
          width: '100%',
          margin: '0 auto',
        }}>
          {children}
        </div>
      </main>
    </div>
  );
}
