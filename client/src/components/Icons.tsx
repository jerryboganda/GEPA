import React from 'react';

export const IconCheck: React.FC<{ className?: string }> = ({ className = 'w-5 h-5' }) => (
  <svg width="20" height="20" style={{ maxWidth: '100%', display: 'inline-block', flexShrink: 0 }} className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5} aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
  </svg>
);

export const IconAlert: React.FC<{ className?: string }> = ({ className = 'w-5 h-5' }) => (
  <svg width="20" height="20" style={{ maxWidth: '100%', display: 'inline-block', flexShrink: 0 }} className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2} aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
  </svg>
);

export const IconClock: React.FC<{ className?: string }> = ({ className = 'w-5 h-5' }) => (
  <svg width="20" height="20" style={{ maxWidth: '100%', display: 'inline-block', flexShrink: 0 }} className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2} aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
  </svg>
);

export const IconPlay: React.FC<{ className?: string }> = ({ className = 'w-5 h-5' }) => (
  <svg width="20" height="20" style={{ maxWidth: '100%', display: 'inline-block', flexShrink: 0 }} className={className} fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
    <path d="M8 5v14l11-7z" />
  </svg>
);

export const IconPause: React.FC<{ className?: string }> = ({ className = 'w-5 h-5' }) => (
  <svg width="20" height="20" style={{ maxWidth: '100%', display: 'inline-block', flexShrink: 0 }} className={className} fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
    <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
  </svg>
);

export const IconMic: React.FC<{ className?: string }> = ({ className = 'w-5 h-5' }) => (
  <svg width="20" height="20" style={{ maxWidth: '100%', display: 'inline-block', flexShrink: 0 }} className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2} aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 003-3V5a3 3 0 00-6 0v6a3 3 0 003 3z" />
  </svg>
);

export const IconVolume: React.FC<{ className?: string }> = ({ className = 'w-5 h-5' }) => (
  <svg width="20" height="20" style={{ maxWidth: '100%', display: 'inline-block', flexShrink: 0 }} className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2} aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
  </svg>
);

export const IconShield: React.FC<{ className?: string }> = ({ className = 'w-5 h-5' }) => (
  <svg width="20" height="20" style={{ maxWidth: '100%', display: 'inline-block', flexShrink: 0 }} className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2} aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
  </svg>
);

export const IconSettings: React.FC<{ className?: string }> = ({ className = 'w-5 h-5' }) => (
  <svg width="20" height="20" style={{ maxWidth: '100%', display: 'inline-block', flexShrink: 0 }} className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2} aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
    <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
  </svg>
);

export const IconArrowRight: React.FC<{ className?: string }> = ({ className = 'w-5 h-5' }) => (
  <svg width="20" height="20" style={{ maxWidth: '100%', display: 'inline-block', flexShrink: 0 }} className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2} aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" d="M14 5l7 7m0 0l-7 7m7-7H3" />
  </svg>
);

export const IconBook: React.FC<{ className?: string }> = ({ className = 'w-5 h-5' }) => (
  <svg width="20" height="20" style={{ maxWidth: '100%', display: 'inline-block', flexShrink: 0 }} className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2} aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
  </svg>
);

export const IconHeadphones: React.FC<{ className?: string }> = ({ className = 'w-5 h-5' }) => (
  <svg width="20" height="20" style={{ maxWidth: '100%', display: 'inline-block', flexShrink: 0 }} className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2} aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" d="M3 18v-6a9 9 0 0118 0v6M3 18a3 3 0 003 3h1a1 1 0 001-1v-4a1 1 0 00-1-1H4a1 1 0 00-1 1zm18 0a3 3 0 01-3 3h-1a1 1 0 01-1-1v-4a1 1 0 011-1h3a1 1 0 011 1z" />
  </svg>
);

export const IconEdit: React.FC<{ className?: string }> = ({ className = 'w-5 h-5' }) => (
  <svg width="20" height="20" style={{ maxWidth: '100%', display: 'inline-block', flexShrink: 0 }} className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2} aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
  </svg>
);
