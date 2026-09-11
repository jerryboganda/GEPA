import React, { useEffect, useState, useRef } from 'react';
import { IconClock } from './Icons';

interface TimerProps {
  deadlineAt: string;
  onTimeout: () => void;
  className?: string;
}

export const Timer: React.FC<TimerProps> = ({ deadlineAt, onTimeout, className = '' }) => {
  const [secondsRemaining, setSecondsRemaining] = useState<number>(() => {
    const diff = Math.max(0, Math.floor((new Date(deadlineAt).getTime() - Date.now()) / 1000));
    return diff;
  });
  const [announcement, setAnnouncement] = useState<string>('');
  const timeoutFired = useRef(false);

  useEffect(() => {
    timeoutFired.current = false;

    const interval = setInterval(() => {
      const now = Date.now();
      const end = new Date(deadlineAt).getTime();
      const diff = Math.max(0, Math.floor((end - now) / 1000));

      setSecondsRemaining(diff);

      if (diff === 60) {
        setAnnouncement('60 seconds remaining');
      } else if (diff === 15) {
        setAnnouncement('15 seconds remaining');
      } else if (diff === 0 && !timeoutFired.current) {
        timeoutFired.current = true;
        setAnnouncement('Time is up for this question');
        onTimeout();
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [deadlineAt, onTimeout]);

  const mins = Math.floor(secondsRemaining / 60);
  const secs = secondsRemaining % 60;
  const formatted = `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
  const isUrgent = secondsRemaining <= 15;
  const isWarning = secondsRemaining <= 60 && !isUrgent;

  return (
    <div className={`inline-flex items-center gap-2 ${className}`}>
      {/* Screen Reader Live Region for WCAG 2.2 AA */}
      <div aria-live="assertive" aria-atomic="true" className="sr-only">
        {announcement}
      </div>

      <div
        className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg border font-mono text-sm font-semibold transition-colors ${
          isUrgent
            ? 'bg-rose-50 border-rose-300 text-rose-700 animate-pulse'
            : isWarning
            ? 'bg-amber-50 border-amber-300 text-amber-800'
            : 'bg-slate-50 border-slate-200 text-slate-700'
        }`}
        aria-label={`Time remaining: ${formatted}`}
        data-testid="timer-countdown"
      >
        <IconClock className={`w-4 h-4 ${isUrgent ? 'text-rose-600' : isWarning ? 'text-amber-600' : 'text-slate-500'}`} />
        <span>{formatted}</span>
      </div>
    </div>
  );
};
