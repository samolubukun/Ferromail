import React from 'react';

interface LogoProps {
  size?: number;
}

export default function Logo({ size = 38 }: LogoProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 200 200"
      width={size}
      height={size}
      style={{ display: 'block' }}
    >
      {/* Background card */}
      <rect width="200" height="200" rx="40" fill="#0A0F1D"/>
      
      {/* Brand accent indicator lines (matching brand gradient) */}
      <rect x="20" y="80" width="60" height="10" rx="5" fill="#6366f1"/>
      <rect x="30" y="100" width="50" height="10" rx="5" fill="#4f46e5"/>
      <rect x="25" y="120" width="55" height="10" rx="5" fill="#818cf8"/>
      
      {/* Main envelope */}
      <path
        d="M70 70 L170 70 L170 140 L70 140 Z M70 70 L120 115 L170 70"
        fill="#141E33"
        stroke="#F8FAFC"
        strokeWidth="10"
        strokeLinejoin="round"
        strokeLinecap="round"
      />
      
      {/* Outbound sending arrow */}
      <path d="M120 70 L120 50 L110 60 L130 60 L120 50 Z" fill="#F8FAFC"/>
    </svg>
  );
}
