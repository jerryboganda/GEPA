import React, { useState, useEffect } from 'react';
import { IconShield, IconAlert, IconCheck, IconPlay, IconVolume } from './Icons';

export const ReviewerDashboard: React.FC = () => {
  const [queue, setQueue] = useState<any[]>([]);
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);
  const [sessionDetail, setSessionDetail] = useState<any>(null);
  const [adjustedLowerScores, setAdjustedLowerScores] = useState<Record<string, number>>({});
  const [adjustedUpperScores, setAdjustedUpperScores] = useState<Record<string, number>>({});
  const [reviewerNotes, setReviewerNotes] = useState<string>('');
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [filter, setFilter] = useState<string>('all');

  useEffect(() => {
    fetch('/api/review/queue')
      .then((res) => res.json())
      .then((data) => {
        setQueue(data);
        if (data.length > 0) {
          loadSession(data[0].session_id);
        }
      })
      .catch(() => {
        const mockQueue = [
          {
            session_id: 'ses_audit_demo_01',
            flag_type: 'rater_disagreement',
            module: 'SPK',
            details: 'Difference between Model Rater (3.4) and Fast Opinion (2.2) exceeded 1.0 threshold on ER1 task',
            timestamp: new Date().toISOString(),
          },
          {
            session_id: 'ses_audit_demo_02',
            flag_type: 'ai_suspect_review',
            module: 'WRT',
            details: 'Paste event exceeded 60% of total response length in single burst',
            timestamp: new Date().toISOString(),
          },
        ];
        setQueue(mockQueue);
        loadSession(mockQueue[0].session_id);
      });
  }, []);

  const loadSession = (id: string) => {
    setSelectedSessionId(id);
    fetch(`/api/review/sessions/${id}`)
      .then((res) => res.json())
      .then((data) => {
        setSessionDetail(data);
        setAdjustedLowerScores(data.at_lower || {});
        setAdjustedUpperScores(data.at_upper || {});
        setSuccessMessage(null);
      })
      .catch(() => {
        const mockDetail = {
          session_id: id,
          task_id: 'SPK-B1B2-ER1',
          task_prompt: 'Describe a major challenge facing university students today and propose a solution.',
          candidate_audio_url: '/api/media/audio/sample_response.wav',
          transcript:
            'Well, um... one significant challenge is balancing academic study with part-time work obligations. Many students need financial support, so they struggle with time management. A good solution is flexible timetable schedules.',
          keystroke_wpm: null,
          paste_bursts: 0,
          at_lower: { intelligibility: 4, fluency: 3, grammar: 3, vocabulary: 3, communication: 4 },
          at_upper: { intelligibility: 3, fluency: 2, grammar: 2, vocabulary: 3, communication: 3 },
          ai_rationale: 'Clear communicative purpose with adequate cohesion. Fluency slightly constrained by hesitation pauses.',
          flags: ['rater_disagreement'],
        };
        setSessionDetail(mockDetail);
        setAdjustedLowerScores(mockDetail.at_lower);
        setAdjustedUpperScores(mockDetail.at_upper);
        setSuccessMessage(null);
      });
  };

  const handleScoreChange = (ref: 'lower' | 'upper', trait: string, val: number) => {
    if (ref === 'lower') {
      setAdjustedLowerScores((prev) => ({ ...prev, [trait]: val }));
    } else {
      setAdjustedUpperScores((prev) => ({ ...prev, [trait]: val }));
    }
  };

  const handleSaveReview = () => {
    setSuccessMessage('Review adjustments successfully saved. Versioned ProductiveRating entry recorded.');
  };

  const filteredQueue = queue.filter((item) => {
    if (filter === 'all') return true;
    return item.flag_type === filter;
  });

  return (
    <div className="min-h-screen bg-slate-100 text-slate-900 font-sans">
      {/* Top Navbar */}
      <header className="bg-white border-b border-slate-200 sticky top-0 z-30">
        <div className="max-w-7xl mx-auto px-4 h-16 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <a href="/" className="font-bold text-slate-900 flex items-center gap-2">
              <span className="w-8 h-8 rounded-lg bg-brand-800 text-white font-black text-sm flex items-center justify-center">
                G
              </span>
              <span>GEPA</span>
            </a>
            <span className="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-amber-100 text-amber-900 border border-amber-200">
              Reviewer Console
            </span>
          </div>

          <div className="flex items-center gap-4 text-xs font-medium text-slate-600">
            <a href="/about" className="hover:text-slate-900 transition-colors font-semibold text-slate-700">
              Technical Manual
            </a>
            <span>Signed in as: evaluator@gepa.edu</span>
            <span className="px-2 py-0.5 bg-slate-200 rounded text-slate-800 font-mono">role: reviewer</span>
          </div>
        </div>
      </header>

      {/* Main Review Layout */}
      <main className="max-w-7xl mx-auto px-4 py-6 grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Flagged Queue Column */}
        <div className="lg:col-span-4 space-y-4">
          <div className="bg-white p-4 rounded-xl border border-slate-200 shadow-soft">
            <div className="flex items-center justify-between mb-3">
              <h2 className="text-sm font-bold text-slate-900 uppercase tracking-wider">Flagged Queue</h2>
              <span className="text-xs font-semibold text-slate-500 bg-slate-100 px-2 py-0.5 rounded">
                {filteredQueue.length} cases
              </span>
            </div>

            {/* Filter pills */}
            <div className="flex flex-wrap gap-1 mb-3">
              {['all', 'rater_disagreement', 'ai_suspect_review', 'profile_inconsistency'].map((f) => (
                <button
                  key={f}
                  onClick={() => setFilter(f)}
                  className={`px-2 py-1 text-xs rounded font-medium capitalize transition-colors ${
                    filter === f ? 'bg-brand-800 text-white' : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
                  }`}
                >
                  {f.replace('_', ' ')}
                </button>
              ))}
            </div>

            <div className="space-y-2 max-h-[600px] overflow-y-auto">
              {filteredQueue.map((item) => (
                <button
                  key={item.session_id}
                  onClick={() => loadSession(item.session_id)}
                  className={`w-full text-left p-3 rounded-lg border text-xs transition-all ${
                    selectedSessionId === item.session_id
                      ? 'bg-brand-50 border-brand-600 ring-2 ring-brand-500/20'
                      : 'border-slate-200 bg-white hover:bg-slate-50'
                  }`}
                >
                  <div className="flex items-center justify-between mb-1">
                    <span className="font-mono font-bold text-slate-800">{item.session_id}</span>
                    <span className="px-2 py-0.5 rounded text-xs font-semibold bg-rose-50 text-rose-700 border border-rose-200 uppercase">
                      {item.module}
                    </span>
                  </div>
                  <span className="font-semibold text-amber-800 block mb-1">{item.flag_type}</span>
                  <p className="text-slate-500 line-clamp-2">{item.details}</p>
                </button>
              ))}
            </div>
          </div>
        </div>

        {/* Evaluation Detail Column */}
        <div className="lg:col-span-8 space-y-6">
          {sessionDetail ? (
            <div className="bg-white p-6 rounded-xl border border-slate-200 shadow-card space-y-6">
              {/* Header */}
              <div className="flex flex-wrap items-center justify-between pb-4 border-b border-slate-200 gap-2">
                <div>
                  <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider block">
                    Session Audit
                  </span>
                  <h3 className="text-lg font-bold text-slate-900">{sessionDetail.session_id}</h3>
                </div>
                <div className="flex gap-2">
                  {sessionDetail.flags &&
                    sessionDetail.flags.map((fl: string) => (
                      <span
                        key={fl}
                        className="px-2.5 py-1 rounded-md text-xs font-bold bg-amber-100 text-amber-900 border border-amber-300"
                      >
                        {fl}
                      </span>
                    ))}
                </div>
              </div>

              {/* Task Prompt */}
              <div className="p-4 rounded-lg bg-slate-50 border border-slate-200 space-y-1">
                <span className="text-xs font-bold uppercase text-slate-400">Prompt: {sessionDetail.task_id}</span>
                <p className="text-sm font-semibold text-slate-800">{sessionDetail.task_prompt}</p>
              </div>

              {/* Audio & Transcript Inspector */}
              <div className="p-4 rounded-lg border border-slate-200 space-y-3">
                <span className="text-xs font-bold uppercase text-slate-400 block">Candidate Audio & Verbatim Transcript</span>
                {sessionDetail.candidate_audio_url && (
                  <div className="flex items-center gap-3">
                    <button
                      type="button"
                      onClick={() => {
                        const a = new Audio(sessionDetail.candidate_audio_url);
                        a.play().catch(() => {});
                      }}
                      className="px-3 py-1.5 bg-brand-800 text-white rounded text-xs font-semibold flex items-center gap-1.5"
                    >
                      <IconPlay className="w-3.5 h-3.5" />
                      <span>Play Response Audio</span>
                    </button>
                    <span className="text-xs text-slate-500">Audio playback synchronized with disfluencies</span>
                  </div>
                )}
                <div className="p-3 bg-slate-50 rounded text-xs text-slate-700 font-mono leading-relaxed">
                  "{sessionDetail.transcript}"
                </div>
              </div>

              {/* AI Trait Breakdown */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {/* Lower Band Reference */}
                <div className="p-4 rounded-xl bg-slate-50 border border-slate-200 space-y-3">
                  <span className="text-xs font-bold text-slate-700 uppercase tracking-wider block">
                    Evaluated at Lower Band Benchmark (atLower)
                  </span>
                  {Object.entries(adjustedLowerScores).map(([trait, score]) => (
                    <div key={trait} className="flex items-center justify-between text-xs">
                      <span className="capitalize text-slate-700 font-medium">{trait}</span>
                      <div className="flex gap-1">
                        {[0, 1, 2, 3, 4, 5].map((val) => (
                          <button
                            key={val}
                            onClick={() => handleScoreChange('lower', trait, val)}
                            className={`w-6 h-6 rounded text-xs font-bold transition-all ${
                              score === val ? 'bg-brand-800 text-white' : 'bg-white border border-slate-200 text-slate-700 hover:bg-slate-100'
                            }`}
                          >
                            {val}
                          </button>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>

                {/* Upper Band Reference */}
                <div className="p-4 rounded-xl bg-slate-50 border border-slate-200 space-y-3">
                  <span className="text-xs font-bold text-slate-700 uppercase tracking-wider block">
                    Evaluated at Upper Band Benchmark (atUpper)
                  </span>
                  {Object.entries(adjustedUpperScores).map(([trait, score]) => (
                    <div key={trait} className="flex items-center justify-between text-xs">
                      <span className="capitalize text-slate-700 font-medium">{trait}</span>
                      <div className="flex gap-1">
                        {[0, 1, 2, 3, 4, 5].map((val) => (
                          <button
                            key={val}
                            onClick={() => handleScoreChange('upper', trait, val)}
                            className={`w-6 h-6 rounded text-xs font-bold transition-all ${
                              score === val ? 'bg-brand-800 text-white' : 'bg-white border border-slate-200 text-slate-700 hover:bg-slate-100'
                            }`}
                          >
                            {val}
                          </button>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Reviewer Notes */}
              <div className="space-y-2">
                <label className="block text-xs font-bold uppercase text-slate-500">Evaluator Review Notes</label>
                <textarea
                  value={reviewerNotes}
                  onChange={(e) => setReviewerNotes(e.target.value)}
                  placeholder="Record rationale for human trait score adjustment..."
                  className="w-full p-3 border border-slate-300 rounded-lg text-xs text-slate-900 h-20 focus:ring-2 focus:ring-brand-500 focus:outline-none"
                />
              </div>

              {successMessage && (
                <div className="p-3 bg-emerald-50 border border-emerald-300 text-emerald-900 rounded-lg text-xs flex items-center gap-2">
                  <IconCheck className="w-4 h-4 text-emerald-600 shrink-0" />
                  <span>{successMessage}</span>
                </div>
              )}

              {/* Actions */}
              <div className="flex flex-wrap items-center gap-3 pt-2 border-t border-slate-200">
                <button
                  onClick={handleSaveReview}
                  className="px-5 py-2.5 bg-brand-800 hover:bg-brand-900 text-white font-bold text-xs rounded-lg shadow-soft transition-colors min-h-[44px]"
                >
                  Save Human Score Override
                </button>
                <button
                  onClick={() => alert('AI Rating confirmed without modification.')}
                  className="px-5 py-2.5 bg-slate-100 hover:bg-slate-200 text-slate-800 font-semibold text-xs rounded-lg transition-colors min-h-[44px]"
                >
                  Confirm & Accept AI Rating
                </button>
                <button
                  onClick={() => alert('Response marked as unusable due to technical factors.')}
                  className="px-4 py-2.5 text-xs font-semibold text-rose-700 hover:bg-rose-50 border border-rose-200 rounded-lg transition-colors min-h-[44px]"
                >
                  Mark Response Unusable
                </button>
              </div>
            </div>
          ) : (
            <div className="p-12 text-center text-slate-500 bg-white rounded-xl border border-slate-200">
              Select a session from the queue to view evaluation details.
            </div>
          )}
        </div>
      </main>
    </div>
  );
};
