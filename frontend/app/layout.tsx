import './globals.css';
import React from 'react';

export const metadata = {
  title: 'Ferromail Dashboard',
  description: 'The Next-Gen Open-Source Email Platform powered by Rust and Next.js',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
