import React, { useState, useEffect, useRef } from 'react';

interface WritingEditorProps {
  initialValue?: string;
  wordGuidance?: { min: number; max: number } | null;
  onAutosave?: (text: string) => void;
  onChange?: (text: string) => void;
  className?: string;
}

export const WritingEditor: React.FC<WritingEditorProps> = ({
  initialValue = '',
  wordGuidance,
  onAutosave,
  onChange,
  className = '',
}) => {
  const [text, setText] = useState<string>(initialValue);
  const [lastSaved, setLastSaved] = useState<string>('All changes saved');
  const [pasteCount, setPasteCount] = useState<number>(0);
  const [focusLossCount, setFocusLossCount] = useState<number>(0);

  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const wordCount = text
    .trim()
    .split(/\s+/)
    .filter((w) => w.length > 0).length;

  const isBelowMin = wordGuidance ? wordCount < wordGuidance.min : false;
  const isAboveMax = wordGuidance ? wordCount > wordGuidance.max : false;
  const isOptimal = wordGuidance ? !isBelowMin && !isAboveMax : true;

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const val = e.target.value;
    setText(val);
    if (onChange) onChange(val);

    setLastSaved('Saving draft...');
    if (timeoutRef.current) clearTimeout(timeoutRef.current);

    timeoutRef.current = setTimeout(() => {
      if (onAutosave) onAutosave(val);
      setLastSaved('Saved seconds ago');
    }, 2000);
  };

  const handlePaste = () => {
    setPasteCount((p) => p + 1);
  };

  const handleBlur = () => {
    setFocusLossCount((f) => f + 1);
    if (onAutosave) onAutosave(text);
    setLastSaved('Saved seconds ago');
  };

  return (
    <div className={`flex flex-col h-full bg-white rounded-xl border border-slate-200 shadow-soft ${className}`}>
      {/* Header telemetry and word guidance */}
      <div className="flex flex-wrap items-center justify-between gap-3 px-4 py-3 bg-slate-50/80 border-b border-slate-200 rounded-t-xl text-xs">
        <div className="flex items-center gap-3">
          {/* Word Guidance Badge */}
          {wordGuidance ? (
            <div
              className={`px-2.5 py-1 rounded-full font-semibold border flex items-center gap-1.5 transition-colors ${
                isOptimal
                  ? 'bg-emerald-50 border-emerald-300 text-emerald-800'
                  : 'bg-amber-50 border-amber-300 text-amber-800'
              }`}
            >
              <span data-testid="writing-word-count">{wordCount} words</span>
              <span className="text-slate-400 font-normal">|</span>
              <span className="font-normal text-slate-600">
                Aim for {wordGuidance.min}–{wordGuidance.max} words
              </span>
            </div>
          ) : (
            <div className="font-medium text-slate-700 bg-white px-2.5 py-1 rounded-md border border-slate-200">
              {wordCount} words
            </div>
          )}
        </div>

        <div className="flex items-center gap-4 text-slate-500">
          <span className="text-xs font-mono">{lastSaved}</span>
        </div>
      </div>

      {/* Editor Area */}
      <div className="flex-1 p-4">
        <textarea
          data-testid="writing-textarea"
          value={text}
          onChange={handleChange}
          onPaste={handlePaste}
          onBlur={handleBlur}
          spellCheck={false}
          autoComplete="off"
          autoCorrect="off"
          autoCapitalize="off"
          placeholder="Compose your response here..."
          className="w-full h-full min-h-[260px] p-2 text-base text-slate-900 placeholder:text-slate-400 border-0 focus:outline-none focus:ring-0 resize-none font-sans leading-relaxed"
        />
      </div>

      {/* Subtle bottom note */}
      <div className="px-4 py-2 bg-slate-50 border-t border-slate-100 rounded-b-xl flex items-center justify-between text-xs text-slate-400">
        <span>Spellcheck disabled for diagnostic evaluation</span>
        <span>Autosave active</span>
      </div>
    </div>
  );
};
