import React, { useState, useEffect } from 'react';
import { IconSettings, IconCheck } from './Icons';

export interface AccessSettings {
  textScale: number;
  spacing: 'normal' | 'wide';
  highContrast: boolean;
  extendedTime: boolean;
  transcriptAccess: boolean;
  oralReadingAlternative: boolean;
}

const DEFAULT_SETTINGS: AccessSettings = {
  textScale: 100,
  spacing: 'normal',
  highContrast: false,
  extendedTime: false,
  transcriptAccess: false,
  oralReadingAlternative: false,
};

export const AccessibilityDrawer: React.FC<{
  onUpdate?: (settings: AccessSettings) => void;
}> = ({ onUpdate }) => {
  const [isOpen, setIsOpen] = useState(false);
  const [settings, setSettings] = useState<AccessSettings>(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('gepa_access_settings');
      if (saved) {
        try {
          return JSON.parse(saved);
        } catch {
          return DEFAULT_SETTINGS;
        }
      }
    }
    return DEFAULT_SETTINGS;
  });

  useEffect(() => {
    if (typeof document !== 'undefined') {
      const root = document.documentElement;
      root.style.fontSize = `${settings.textScale}%`;

      if (settings.spacing === 'wide') {
        root.classList.add('tracking-wide', 'leading-loose');
      } else {
        root.classList.remove('tracking-wide', 'leading-loose');
      }

      if (settings.highContrast) {
        root.classList.add('contrast-more', 'high-contrast');
      } else {
        root.classList.remove('contrast-more', 'high-contrast');
      }

      localStorage.setItem('gepa_access_settings', JSON.stringify(settings));
    }
    if (onUpdate) {
      onUpdate(settings);
    }
  }, [settings, onUpdate]);

  const toggleOpen = () => setIsOpen((prev) => !prev);

  return (
    <>
      <button
        onClick={toggleOpen}
        aria-label="Open Display and Access Settings"
        className="fixed top-4 right-4 z-40 flex items-center gap-2 px-3 py-2 text-sm font-medium text-slate-700 bg-white border border-slate-300 rounded-lg shadow-soft hover:bg-slate-50 focus:outline-none focus:ring-4 focus:ring-brand-500/20 active:bg-slate-100 transition-colors"
      >
        <IconSettings className="w-4 h-4 text-slate-600" />
        <span>Display & Access</span>
      </button>

      {isOpen && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="access-drawer-title"
          className="fixed inset-0 z-50 overflow-hidden bg-slate-900/40 backdrop-blur-xs flex justify-end"
        >
          <div className="w-full max-w-md bg-white h-full shadow-elevated flex flex-col justify-between overflow-y-auto p-6 animate-in slide-in-from-right duration-200">
            <div>
              <div className="flex items-center justify-between pb-4 border-b border-slate-200">
                <h2 id="access-drawer-title" className="text-lg font-semibold text-slate-900">
                  Display & Access Preferences
                </h2>
                <button
                  onClick={toggleOpen}
                  aria-label="Close preferences"
                  className="p-1 text-slate-500 hover:text-slate-800 rounded-md focus:ring-2 focus:ring-brand-500"
                >
                  <span className="text-xl leading-none">&times;</span>
                </button>
              </div>

              <div className="py-4 space-y-6">
                {/* Text Scaling */}
                <div>
                  <label className="block text-sm font-medium text-slate-900 mb-2">
                    Text Size: <span className="font-semibold text-brand-700">{settings.textScale}%</span>
                  </label>
                  <div className="grid grid-cols-4 gap-2">
                    {[100, 125, 150, 200].map((scale) => (
                      <button
                        key={scale}
                        onClick={() => setSettings((s) => ({ ...s, textScale: scale }))}
                        className={`py-2 text-sm font-medium rounded-lg border transition-all ${
                          settings.textScale === scale
                            ? 'bg-brand-50 border-brand-600 text-brand-800 ring-2 ring-brand-500/20'
                            : 'border-slate-200 hover:bg-slate-50 text-slate-700'
                        }`}
                      >
                        {scale}%
                      </button>
                    ))}
                  </div>
                </div>

                {/* Spacing */}
                <div>
                  <label className="block text-sm font-medium text-slate-900 mb-2">Line & Letter Spacing</label>
                  <div className="grid grid-cols-2 gap-2">
                    {(['normal', 'wide'] as const).map((mode) => (
                      <button
                        key={mode}
                        onClick={() => setSettings((s) => ({ ...s, spacing: mode }))}
                        className={`py-2 text-sm font-medium rounded-lg border capitalize transition-all ${
                          settings.spacing === mode
                            ? 'bg-brand-50 border-brand-600 text-brand-800 ring-2 ring-brand-500/20'
                            : 'border-slate-200 hover:bg-slate-50 text-slate-700'
                        }`}
                      >
                        {mode === 'normal' ? 'Standard' : 'Spacious (1.8x)'}
                      </button>
                    ))}
                  </div>
                </div>

                {/* High Contrast */}
                <div className="flex items-center justify-between p-3 rounded-lg border border-slate-200 bg-slate-50">
                  <div>
                    <span className="text-sm font-medium text-slate-900 block">High Contrast Mode</span>
                    <span className="text-xs text-slate-500">Enhanced 7:1 border and text contrast</span>
                  </div>
                  <input
                    type="checkbox"
                    checked={settings.highContrast}
                    onChange={(e) => setSettings((s) => ({ ...s, highContrast: e.target.checked }))}
                    className="w-5 h-5 accent-brand-600 rounded cursor-pointer"
                  />
                </div>

                {/* Extended Time */}
                <div className="flex items-center justify-between p-3 rounded-lg border border-slate-200 bg-slate-50">
                  <div>
                    <span className="text-sm font-medium text-slate-900 block">Extended Time (1.5×)</span>
                    <span className="text-xs text-slate-500">Self-declared accommodation across all timers</span>
                  </div>
                  <input
                    type="checkbox"
                    checked={settings.extendedTime}
                    onChange={(e) => setSettings((s) => ({ ...s, extendedTime: e.target.checked }))}
                    className="w-5 h-5 accent-brand-600 rounded cursor-pointer"
                  />
                </div>

                {/* Transcript Access for Listening */}
                <div className="flex items-center justify-between p-3 rounded-lg border border-slate-200 bg-slate-50">
                  <div>
                    <span className="text-sm font-medium text-slate-900 block">Transcript Access Mode</span>
                    <span className="text-xs text-slate-500">
                      Shows listening transcripts. Listening skill reported as not measured.
                    </span>
                  </div>
                  <input
                    type="checkbox"
                    checked={settings.transcriptAccess}
                    onChange={(e) => setSettings((s) => ({ ...s, transcriptAccess: e.target.checked }))}
                    className="w-5 h-5 accent-brand-600 rounded cursor-pointer"
                  />
                </div>

                {/* Speech Difference Alternative */}
                <div className="flex items-center justify-between p-3 rounded-lg border border-slate-200 bg-slate-50">
                  <div>
                    <span className="text-sm font-medium text-slate-900 block">Speech Difference Pathway</span>
                    <span className="text-xs text-slate-500">
                      Replaces Oral Reading with an additional Functional Situation task.
                    </span>
                  </div>
                  <input
                    type="checkbox"
                    checked={settings.oralReadingAlternative}
                    onChange={(e) => setSettings((s) => ({ ...s, oralReadingAlternative: e.target.checked }))}
                    className="w-5 h-5 accent-brand-600 rounded cursor-pointer"
                  />
                </div>
              </div>
            </div>

            <div className="pt-4 border-t border-slate-200">
              <button
                onClick={toggleOpen}
                className="w-full py-2.5 px-4 bg-brand-800 text-white font-medium text-sm rounded-lg hover:bg-brand-900 focus:outline-none focus:ring-4 focus:ring-brand-500/20 transition-colors shadow-soft"
              >
                Apply & Return to Assessment
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
};
