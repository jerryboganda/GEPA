import React, { useState, useRef, useEffect } from 'react';
import { IconPlay, IconVolume, IconCheck } from './Icons';

interface AudioPlayerProps {
  audioUrl?: string;
  onPlayStarted?: () => void;
  onPlayCompleted?: (playCount: number) => void;
  className?: string;
}

export const AudioPlayer: React.FC<AudioPlayerProps> = ({
  audioUrl,
  onPlayStarted,
  onPlayCompleted,
  className = '',
}) => {
  const [playCount, setPlayCount] = useState<number>(0);
  const [isPlaying, setIsPlaying] = useState<boolean>(false);
  const [progress, setProgress] = useState<number>(0);
  const audioRef = useRef<HTMLAudioElement | null>(null);

  useEffect(() => {
    // Reset on audioUrl change
    setPlayCount(0);
    setIsPlaying(false);
    setProgress(0);
  }, [audioUrl]);

  const handlePlay = () => {
    if (playCount >= 2 || isPlaying) return;

    if (onPlayStarted) {
      onPlayStarted();
    }

    setIsPlaying(true);
    const audio = audioRef.current;
    if (audio) {
      audio.currentTime = 0;
      audio.play().catch(() => {
        // Fallback for mock/test environments
        simulateAudioPlayback();
      });
    } else {
      simulateAudioPlayback();
    }
  };

  const simulateAudioPlayback = () => {
    let current = 0;
    const durationMs = 5000; // 5 second mock playback
    const step = 100;
    const interval = setInterval(() => {
      current += step;
      const pct = Math.min(100, Math.round((current / durationMs) * 100));
      setProgress(pct);

      if (current >= durationMs) {
        clearInterval(interval);
        finishPlayback();
      }
    }, step);
  };

  const finishPlayback = () => {
    setIsPlaying(false);
    setProgress(100);
    const newCount = playCount + 1;
    setPlayCount(newCount);
    if (onPlayCompleted) {
      onPlayCompleted(newCount);
    }
  };

  const isLocked = playCount >= 2 || isPlaying;

  return (
    <div className={`p-4 rounded-xl border border-slate-200 bg-slate-50/80 shadow-soft ${className}`}>
      {audioUrl && (
        <audio
          ref={audioRef}
          src={audioUrl}
          onTimeUpdate={() => {
            const el = audioRef.current;
            if (el && el.duration) {
              setProgress(Math.round((el.currentTime / el.duration) * 100));
            }
          }}
          onEnded={finishPlayback}
          className="hidden"
          preload="auto"
        />
      )}

      <div className="flex flex-col sm:flex-row items-center justify-between gap-4">
        <div className="flex items-center gap-3 w-full sm:w-auto">
          <div className="w-10 h-10 rounded-full bg-brand-100 text-brand-700 flex items-center justify-center shrink-0">
            <IconVolume className="w-5 h-5" />
          </div>
          <div>
            <div className="text-sm font-semibold text-slate-900">
              {isPlaying
                ? 'Audio is playing...'
                : playCount === 0
                ? 'Audio Stimulus (Required)'
                : playCount === 1
                ? 'First play complete'
                : 'Playback limit reached'}
            </div>
            <div className="text-xs text-slate-500">
              Non-scrubbable • Maximum 2 plays allowed (never penalised)
            </div>
          </div>
        </div>

        <div className="flex items-center gap-3 w-full sm:w-auto">
          <button
            onClick={handlePlay}
            disabled={isLocked}
            aria-label={
              playCount === 0
                ? 'Play audio stimulus 1 of 2'
                : playCount === 1
                ? 'Play audio stimulus again 2 of 2'
                : 'Audio plays completed'
            }
            className={`w-full sm:w-auto flex items-center justify-center gap-2 px-5 py-2.5 rounded-lg text-sm font-semibold transition-all shadow-soft min-h-[44px] ${
              isLocked
                ? 'bg-slate-200 text-slate-400 cursor-not-allowed'
                : 'bg-brand-700 hover:bg-brand-800 text-white focus:ring-4 focus:ring-brand-500/20 active:bg-brand-900'
            }`}
          >
            {isPlaying ? (
              <>
                <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                <span>Playing ({progress}%)</span>
              </>
            ) : playCount === 0 ? (
              <>
                <IconPlay className="w-4 h-4" />
                <span>Play (1 of 2)</span>
              </>
            ) : playCount === 1 ? (
              <>
                <IconPlay className="w-4 h-4" />
                <span>Play again (2 of 2)</span>
              </>
            ) : (
              <>
                <IconCheck className="w-4 h-4 text-emerald-600" />
                <span>Played (2 of 2)</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Progress Bar (Visual only, non-scrubbable) */}
      {isPlaying && (
        <div className="mt-3 w-full bg-slate-200 rounded-full h-1.5 overflow-hidden">
          <div
            className="bg-brand-600 h-1.5 rounded-full transition-all duration-100 ease-linear"
            style={{ width: `${progress}%` }}
          />
        </div>
      )}
    </div>
  );
};
