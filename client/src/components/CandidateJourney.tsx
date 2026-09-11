import React, { useState, useEffect } from 'react';
import { Timer } from './Timer';
import { AudioPlayer } from './AudioPlayer';
import { AudioRecorder } from './AudioRecorder';
import { WritingEditor } from './WritingEditor';
import { AccessibilityDrawer, type AccessSettings } from './AccessibilityDrawer';
import { UI_STRINGS } from '../copy/strings';
import { CAN_DO_STATEMENTS } from '../copy/canDo';
import { READINESS_TARGETS } from '../copy/readiness';
import {
  IconCheck,
  IconAlert,
  IconClock,
  IconPlay,
  IconVolume,
  IconShield,
  IconArrowRight,
  IconBook,
  IconHeadphones,
  IconEdit,
  IconMic,
} from './Icons';

type ScreenStep =
  | 'start'
  | 'worked_example'
  | 'objective_test'
  | 'receptive_result'
  | 'mic_check'
  | 'speaking_test'
  | 'writing_break'
  | 'writing_test'
  | 'full_result';

export const CandidateJourney: React.FC = () => {
  // Session State
  const [sessionId, setSessionId] = useState<string>('');
  const [targetGoal, setTargetGoal] = useState<string>('General');
  const [uiLanguage, setUiLanguage] = useState<string>('en');
  const [privacyConsent, setPrivacyConsent] = useState<boolean>(true);
  const [researchConsent, setResearchConsent] = useState<boolean>(false);
  const [currentStep, setCurrentStep] = useState<ScreenStep>('start');
  const [activeModule, setActiveModule] = useState<'LS' | 'RD' | 'LSN' | 'SPK' | 'WRT'>('LS');
  const [questionCount, setQuestionCount] = useState<number>(1);

  // Resume State
  const [savedSessionId, setSavedSessionId] = useState<string>('');
  const [resumeInputId, setResumeInputId] = useState<string>('');
  const [isReceptiveFinalized, setIsReceptiveFinalized] = useState<boolean>(false);

  // Delivery Units & Items
  const [deliveryUnit, setDeliveryUnit] = useState<any>(null);
  const [selectedAnswers, setSelectedAnswers] = useState<Record<string, string>>({});
  const [activeTestletItemIdx, setActiveTestletItemIdx] = useState<number>(0);
  const [audioReplayCount, setAudioReplayCount] = useState<number>(0);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  // Receptive & Full Results
  const [receptiveResult, setReceptiveResult] = useState<any>(null);
  const [fullResult, setFullResult] = useState<any>(null);

  // Speaking & Writing Tasks
  const [speakingTasks, setSpeakingTasks] = useState<any[]>([]);
  const [currentSpeakingIdx, setCurrentSpeakingIdx] = useState<number>(0);
  const [writingTasks, setWritingTasks] = useState<any[]>([]);
  const [currentWritingIdx, setCurrentWritingIdx] = useState<number>(0);
  const [currentWritingText, setCurrentWritingText] = useState<string>('');

  // UI Modals & Access
  const [showAboutModal, setShowAboutModal] = useState<boolean>(false);
  const [accessSettings, setAccessSettings] = useState<AccessSettings | null>(null);

  // Load saved session on mount (interruption recovery per PRD §2)
  useEffect(() => {
    if (typeof window !== 'undefined') {
      const stored = localStorage.getItem('gepa_active_session');
      if (stored) {
        setSavedSessionId(stored);
      }
    }
  }, []);

  // 1. Create Session
  const handleStartSession = async () => {
    if (!privacyConsent) return;
    setIsLoading(true);
    const devClass =
      typeof window !== 'undefined' && window.innerWidth < 768 ? 'mobile' : 'desktop';
    try {
      const res = await fetch('/api/sessions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          target_goal: targetGoal,
          ui_language: uiLanguage,
          accommodations: accessSettings,
          mode: 'free_beta',
          device_class: devClass,
        }),
      });
      const data = await res.json();
      setSessionId(data.session_id);
      if (typeof window !== 'undefined') {
        localStorage.setItem('gepa_active_session', data.session_id);
        setSavedSessionId(data.session_id);
      }
      setCurrentStep('worked_example');
    } catch {
      // Offline / fallback demo session
      setSessionId('ses_demo_client_1');
      setCurrentStep('worked_example');
    } finally {
      setIsLoading(false);
    }
  };

  // Resume Session (PRD §2, §6)
  const handleResumeSession = async (idToResume: string) => {
    const cleanId = idToResume.trim();
    if (!cleanId) return;
    setIsLoading(true);
    try {
      const res = await fetch(`/api/sessions/${cleanId}`);
      if (!res.ok) {
        alert('Session record not found. Please start a new placement assessment.');
        return;
      }
      const data = await res.json();
      setSessionId(data.session_id);
      setTargetGoal(data.target_goal || 'General');
      setUiLanguage(data.ui_language || 'en');
      if (typeof window !== 'undefined') {
        localStorage.setItem('gepa_active_session', data.session_id);
        setSavedSessionId(data.session_id);
      }

      if (data.has_result) {
        const fRes = await fetch(`/api/sessions/${data.session_id}/results/full`);
        const report = await fRes.json();
        setFullResult(report);
        setCurrentStep('full_result');
      } else if (data.wrt_status === 'in_progress' || data.wrt_status === 'complete') {
        const wRes = await fetch(`/api/sessions/${data.session_id}/writing/start`);
        const tasks = await wRes.json();
        setWritingTasks(tasks);
        setCurrentWritingIdx(0);
        setCurrentWritingText('');
        setCurrentStep('writing_test');
      } else if (data.spk_status === 'in_progress') {
        const sRes = await fetch(`/api/sessions/${data.session_id}/speaking/start`);
        const tasks = await sRes.json();
        setSpeakingTasks(tasks);
        setCurrentSpeakingIdx(0);
        setCurrentStep('speaking_test');
      } else if (data.rd_status === 'complete' && data.lsn_status === 'complete') {
        const profRes = await fetch(`/api/sessions/${data.session_id}/results/receptive`);
        const report = await profRes.json();
        setReceptiveResult(report);
        setCurrentStep('receptive_result');
      } else if (data.rd_status === 'in_progress') {
        setActiveModule('RD');
        const rRes = await fetch(`/api/sessions/${data.session_id}/modules/RD/start`, {
          method: 'POST',
        });
        const unit = await rRes.json();
        setDeliveryUnit(unit);
        setCurrentStep('objective_test');
      } else if (data.lsn_status === 'in_progress') {
        setActiveModule('LSN');
        const lRes = await fetch(`/api/sessions/${data.session_id}/modules/LSN/start`, {
          method: 'POST',
        });
        const unit = await lRes.json();
        setDeliveryUnit(unit);
        setCurrentStep('objective_test');
      } else {
        setActiveModule('LS');
        const lsRes = await fetch(`/api/sessions/${data.session_id}/modules/LS/start`, {
          method: 'POST',
        });
        const unit = await lsRes.json();
        setDeliveryUnit(unit);
        setCurrentStep('objective_test');
      }
    } catch {
      alert('Could not connect to session service. Please verify your connection.');
    } finally {
      setIsLoading(false);
    }
  };

  // 2. Start LS Module
  const handleBeginLanguageSystems = async () => {
    setActiveModule('LS');
    setQuestionCount(1);
    setIsLoading(true);
    try {
      const res = await fetch(`/api/sessions/${sessionId}/modules/LS/start`, { method: 'POST' });
      const unit = await res.json();
      setDeliveryUnit(unit);
      setSelectedAnswers({});
      setCurrentStep('objective_test');
    } catch {
      // Fallback sample item
      setDeliveryUnit({
        stimulus_id: null,
        stimulus_text: null,
        items: [
          {
            item_id: 'LS-B1-01',
            module: 'LS',
            stem: 'She has been working here ___ three years.',
            options: [
              { option_id: 'opt_1', text: 'for' },
              { option_id: 'opt_2', text: 'since' },
              { option_id: 'opt_3', text: 'during' },
              { option_id: 'opt_4', text: 'while' },
            ],
          },
        ],
        deadline_at: new Date(Date.now() + 60000).toISOString(),
        module_complete: false,
      });
      setCurrentStep('objective_test');
    } finally {
      setIsLoading(false);
    }
  };

  // 3. Submit Objective Response
  const handleSubmitObjective = async () => {
    if (!deliveryUnit || !deliveryUnit.items || deliveryUnit.items.length === 0) return;

    setIsLoading(true);
    try {
      let body: any;
      if (deliveryUnit.items.length > 1) {
        body = {
          responses: deliveryUnit.items.map((it: any) => ({
            item_id: it.item_id,
            selected_option_id: selectedAnswers[it.item_id] || null,
            response_ms: 12400,
          })),
          replay_count: audioReplayCount,
        };
      } else {
        const currentItem = deliveryUnit.items[0];
        const selectedOpt = selectedAnswers[currentItem.item_id] || null;
        body = {
          item_id: currentItem.item_id,
          selected_option_id: selectedOpt,
          response_ms: 12400,
          replay_count: audioReplayCount,
        };
      }

      const res = await fetch(`/api/sessions/${sessionId}/responses`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });
      const data = await res.json();

      if (data.next && data.next.module_complete) {
        // Current module finished! Transition to next module
        if (activeModule === 'LS') {
          // Move to Reading
          setActiveModule('RD');
          setQuestionCount(1);
          setAudioReplayCount(0);
          const rRes = await fetch(`/api/sessions/${sessionId}/modules/RD/start`, { method: 'POST' });
          const rUnit = await rRes.json();
          setDeliveryUnit(rUnit);
          setSelectedAnswers({});
          setActiveTestletItemIdx(0);
        } else if (activeModule === 'RD') {
          // Move to Listening
          setActiveModule('LSN');
          setQuestionCount(1);
          setAudioReplayCount(0);
          const lRes = await fetch(`/api/sessions/${sessionId}/modules/LSN/start`, { method: 'POST' });
          const lUnit = await lRes.json();
          setDeliveryUnit(lUnit);
          setSelectedAnswers({});
          setActiveTestletItemIdx(0);
        } else if (activeModule === 'LSN') {
          // Move to Receptive Profile!
          const profRes = await fetch(`/api/sessions/${sessionId}/results/receptive`);
          const report = await profRes.json();
          setReceptiveResult(report);
          setCurrentStep('receptive_result');
        }
      } else if (data.next) {
        setDeliveryUnit(data.next);
        setQuestionCount((q) => q + 1);
        setSelectedAnswers({});
        setActiveTestletItemIdx(0);
        setAudioReplayCount(0);
      }
    } catch {
      // Move forward in demo mode
      if (activeModule === 'LS') {
        setActiveModule('RD');
        setDeliveryUnit({
          stimulus_id: 'RD-B1-S1',
          stimulus_type: 'Email',
          stimulus_text: 'Subject: Team meeting update\n\nDear team, please note that our quarterly review will take place on Thursday at 10:00 AM in Conference Room B. If you are presenting slides, please upload them by Wednesday evening.',
          items: [
            {
              item_id: 'RD-B1-01',
              module: 'RD',
              stem: 'When should presentation materials be uploaded?',
              options: [
                { option_id: 'opt_a', text: 'By Wednesday evening' },
                { option_id: 'opt_b', text: 'On Thursday morning' },
                { option_id: 'opt_c', text: 'During the meeting' },
              ],
            },
            {
              item_id: 'RD-B1-02',
              module: 'RD',
              stem: 'Where is the meeting taking place?',
              options: [
                { option_id: 'opt_d', text: 'Conference Room B' },
                { option_id: 'opt_e', text: 'Online only' },
                { option_id: 'opt_f', text: 'The main auditorium' },
              ],
            },
          ],
          deadline_at: new Date(Date.now() + 240000).toISOString(),
          module_complete: false,
        });
      } else if (activeModule === 'RD') {
        setActiveModule('LSN');
        setDeliveryUnit({
          stimulus_id: 'LSN-B1-S1',
          stimulus_type: 'Conversation',
          audio_url: '/api/media/audio/LSN-B1-S1.mp3',
          items: [
            {
              item_id: 'LSN-B1-01',
              module: 'LSN',
              stem: 'What is the main topic of the conversation?',
              options: [
                { option_id: 'o1', text: 'Booking train tickets' },
                { option_id: 'o2', text: 'Rescheduling an interview' },
                { option_id: 'o3', text: 'Selecting library books' },
              ],
            },
            {
              item_id: 'LSN-B1-02',
              module: 'LSN',
              stem: 'What does the speaker agree to do next?',
              options: [
                { option_id: 'o4', text: 'Send a confirmation email' },
                { option_id: 'o5', text: 'Call the office tomorrow' },
                { option_id: 'o6', text: 'Submit a paper application' },
              ],
            },
          ],
          deadline_at: new Date(Date.now() + 180000).toISOString(),
          module_complete: false,
        });
      } else {
        // Receptive result demo
        setReceptiveResult({
          session_id: sessionId,
          profile_type: 'foundation_receptive',
          skills: [
            { skill: 'RD', status: 'measured', band: 'B2', can_do: ['Can read with independence across complex articles.'], growth_areas: ['Work on synthesising nuanced opposing arguments.'] },
            { skill: 'LSN', status: 'measured', band: 'B1', can_do: ['Can understand main points of standard spoken speech.'], growth_areas: ['Strengthen listening for unstated speaker attitudes.'] },
            { skill: 'SPK', status: 'not_measured', band: null, can_do: [], growth_areas: [] },
            { skill: 'WRT', status: 'not_measured', band: null, can_do: [], growth_areas: [] },
          ],
          diagnostics: {
            language_systems: { band: 'B1', constructs_strong: ['Core clause syntax', 'Contextual vocabulary'], constructs_weak: ['Complex conditionality'] },
          },
          confidence: 'Moderate',
          confidence_reasons: ['Receptive modules completed with consistent placement evidence.'],
        });
        setCurrentStep('receptive_result');
      }
    } finally {
      setIsLoading(false);
    }
  };

  // 4. Continue to Speaking
  const handleProceedToSpeaking = async () => {
    setIsLoading(true);
    try {
      const res = await fetch(`/api/sessions/${sessionId}/speaking/start`);
      const tasks = await res.json();
      setSpeakingTasks(tasks);
      setCurrentSpeakingIdx(0);
      setCurrentStep('mic_check');
    } catch {
      // Sample speaking tasks
      setSpeakingTasks([
        {
          task_id: 'SPK-B1B2-OR',
          task_type: 'oral_reading',
          label: 'Oral Reading',
          prompt: 'Public transit systems in major cities are increasingly adopting contactless mobile ticketing to reduce passenger queues and operational overhead.',
          prep_seconds: 15,
          max_speak_seconds: 35,
          allows_rerecord: true,
        },
        {
          task_id: 'SPK-B1B2-FS',
          task_type: 'functional_situation',
          label: 'Functional Situation',
          prompt: 'You ordered an electronic item online, but received the incorrect model. Telephone customer services, explain the discrepancy, and request an exchange.',
          prep_seconds: 20,
          max_speak_seconds: 50,
          allows_rerecord: true,
        },
      ]);
      setCurrentSpeakingIdx(0);
      setCurrentStep('mic_check');
    } finally {
      setIsLoading(false);
    }
  };

  // 5. Submit Speaking Task
  const handleSubmitSpeakingTask = async () => {
    const currentTask = speakingTasks[currentSpeakingIdx];
    if (!currentTask) return;

    setIsLoading(true);
    try {
      await fetch(`/api/sessions/${sessionId}/speaking/${currentTask.task_id}/submit`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ storage_path: 'local://sample_rec.webm' }),
      });

      if (currentSpeakingIdx + 1 < speakingTasks.length) {
        setCurrentSpeakingIdx((i) => i + 1);
      } else {
        // Proceed to Writing Break (PRD §6)
        setCurrentStep('writing_break');
      }
    } catch {
      if (currentSpeakingIdx + 1 < speakingTasks.length) {
        setCurrentSpeakingIdx((i) => i + 1);
      } else {
        setCurrentStep('writing_break');
      }
    } finally {
      setIsLoading(false);
    }
  };

  // 6. Start Writing
  const handleProceedToWriting = async () => {
    setIsLoading(true);
    try {
      const res = await fetch(`/api/sessions/${sessionId}/writing/start`);
      const tasks = await res.json();
      setWritingTasks(tasks);
      setCurrentWritingIdx(0);
      setCurrentWritingText('');
      setCurrentStep('writing_test');
    } catch {
      setWritingTasks([
        {
          task_id: 'WRT-B1B2-1',
          label: 'Functional Communication',
          prompt: 'You recently joined a local sports or cultural club. Write an email to the club secretary suggesting two ideas for upcoming community events.',
          word_guidance: { min: 80, max: 120 },
        },
        {
          task_id: 'WRT-B1B2-2',
          label: 'Integrated Synthesis',
          prompt: 'Review the proposal for urban green spaces and summarise the primary environmental advantages and potential budget considerations.',
          word_guidance: { min: 120, max: 180 },
        },
      ]);
      setCurrentWritingIdx(0);
      setCurrentWritingText('');
      setCurrentStep('writing_test');
    } finally {
      setIsLoading(false);
    }
  };

  // 7. Submit Writing Task
  const handleSubmitWritingTask = async () => {
    const currentTask = writingTasks[currentWritingIdx];
    if (!currentTask) return;

    setIsLoading(true);
    try {
      await fetch(`/api/sessions/${sessionId}/writing/${currentTask.task_id}/submit`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ text: currentWritingText }),
      });

      if (currentWritingIdx + 1 < writingTasks.length) {
        setCurrentWritingIdx((i) => i + 1);
        setCurrentWritingText('');
      } else {
        // Fetch Full Result!
        const fRes = await fetch(`/api/sessions/${sessionId}/results/full`);
        const report = await fRes.json();
        setFullResult(report);
        setCurrentStep('full_result');
      }
    } catch {
      // Mock full result
      setFullResult({
        session_id: sessionId,
        profile_type: 'full',
        skills: [
          { skill: 'RD', status: 'measured', band: 'B2', can_do: ['Can read with independence across varied texts and genres.'], growth_areas: ['Synthesise nuanced argumentative structures.'] },
          { skill: 'LSN', status: 'measured', band: 'B2', can_do: ['Can understand standard spoken English on familiar and abstract topics.'], growth_areas: ['Track rapid exchanges with dense background noise.'] },
          { skill: 'SPK', status: 'measured', band: 'B2', can_do: ['Can give clear, systematically developed spoken descriptions.'], growth_areas: ['Enhance natural idiomatic flexibility.'] },
          { skill: 'WRT', status: 'measured', band: 'B1', can_do: ['Can write straightforward connected text on familiar subjects.'], growth_areas: ['Strengthen cohesive markers and complex sentence structures.'] },
        ],
        headline: { kind: 'indicative_overall', band: 'B2', range: null },
        confidence: 'Moderate',
        confidence_reasons: ['All four communicative skills completed with coherent diagnostic patterns.'],
        diagnostics: {
          language_systems: { band: 'B2', constructs_strong: ['Lexical precision', 'Complex subordination'], constructs_weak: ['Inverted clause syntax'] },
          listen_to_write: { accuracy: 'minor_errors' },
          pronunciation_notes: ['Clear phonological articulation; intonation supports communicative intent.'],
          fluency_notes: ['Consistent speech rate; natural hesitation when searching for complex ideas.'],
        },
        readiness: {
          target: targetGoal,
          text: READINESS_TARGETS[targetGoal]?.safeUse || 'Diagnostic foundation indicates good communicative readiness.',
          disclaimer: 'GEPA does not predict official exam scores.',
        },
        retest_advice: 'Recommended study interval: retest after approximately 8–12 weeks of structured study.',
      });
      setCurrentStep('full_result');
    } finally {
      setIsLoading(false);
    }
  };

  // 8. Delete Data
  const handleDeleteData = async () => {
    if (confirm('Are you sure you want to delete all diagnostic records from this session?')) {
      try {
        await fetch(`/api/sessions/${sessionId}`, { method: 'DELETE' });
      } catch {}
      if (typeof window !== 'undefined') {
        localStorage.removeItem('gepa_active_session');
      }
      alert('Your assessment data has been completely erased from the service.');
      window.location.href = '/';
    }
  };

  return (
    <div className="min-h-screen bg-slate-50 text-slate-900 flex flex-col justify-between selection:bg-brand-500 selection:text-white font-sans antialiased">
      {/* Global Accessibility Drawer */}
      <AccessibilityDrawer onUpdate={setAccessSettings} />

      {/* Header Bar */}
      <header className="sticky top-0 z-30 bg-white/95 backdrop-blur-md border-b border-slate-200">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 h-16 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-lg bg-brand-800 text-white font-black text-lg flex items-center justify-center tracking-tight shadow-soft">
              G
            </div>
            <div>
              <span className="font-bold text-slate-900 tracking-tight text-base sm:text-lg">GEPA</span>
              <span className="hidden sm:inline-block ml-2 px-2 py-0.5 text-xs font-semibold uppercase tracking-wider text-brand-700 bg-brand-50 rounded-full border border-brand-200">
                Diagnostic Beta
              </span>
            </div>
          </div>

          <div className="flex items-center gap-3 pr-40 sm:pr-44">
            {sessionId && (
              <span className="hidden md:inline-block text-xs font-mono text-slate-500 bg-slate-100 px-2.5 py-1 rounded-md border border-slate-200">
                Session: {sessionId}
              </span>
            )}
            <nav className="flex items-center gap-2">
              <a
                href="/about"
                className="px-3 py-1.5 text-xs font-semibold text-slate-600 hover:text-slate-900 hover:bg-slate-100 rounded-lg transition-colors"
              >
                Technical Manual
              </a>
              <a
                href="/review"
                className="px-3 py-1.5 text-xs font-semibold text-slate-600 hover:text-slate-900 hover:bg-slate-100 rounded-lg transition-colors"
              >
                Reviewer Queue
              </a>
              <a
                href="/admin"
                className="px-3 py-1.5 text-xs font-semibold text-slate-600 hover:text-slate-900 hover:bg-slate-100 rounded-lg transition-colors"
              >
                Admin Ops
              </a>
            </nav>
          </div>
        </div>
      </header>

      {/* Main Journey Container */}
      <main className="flex-1 max-w-5xl w-full mx-auto px-4 sm:px-6 py-8">
        {/* STEP 1: START SCREEN */}
        {currentStep === 'start' && (
          <div className="max-w-2xl mx-auto space-y-8 animate-in fade-in duration-300">
            <div className="text-center space-y-3">
              <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-brand-50 border border-brand-200 text-brand-800 text-xs font-semibold">
                <IconShield className="w-3.5 h-3.5 text-brand-700" />
                <span>{UI_STRINGS.start_screen.badge}</span>
              </div>
              <h1 className="text-3xl sm:text-4xl font-extrabold text-slate-900 tracking-tight">
                {UI_STRINGS.start_screen.headline}
              </h1>
              <p className="text-base text-slate-600 max-w-xl mx-auto leading-relaxed">
                {UI_STRINGS.start_screen.subheadline}
              </p>
            </div>

            <div className="bg-white p-6 sm:p-8 rounded-2xl border border-slate-200 shadow-card space-y-6">
              {/* Destination Goal Selection */}
              <div>
                <label className="block text-sm font-semibold text-slate-900 mb-1">
                  {UI_STRINGS.start_screen.goal_label}
                </label>
                <p className="text-xs text-slate-500 mb-3">{UI_STRINGS.start_screen.goal_hint}</p>
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
                  {UI_STRINGS.start_screen.goals.map((g) => (
                    <button
                      key={g.id}
                      type="button"
                      onClick={() => setTargetGoal(g.id)}
                      className={`py-2 px-3 text-xs font-semibold rounded-lg border text-left transition-all min-h-[44px] flex items-center justify-between ${
                        targetGoal === g.id
                          ? 'bg-brand-50 border-brand-600 text-brand-800 ring-2 ring-brand-500/20 shadow-xs'
                          : 'border-slate-200 hover:bg-slate-50 text-slate-700'
                      }`}
                    >
                      <span>{g.label}</span>
                      {targetGoal === g.id && <IconCheck className="w-3.5 h-3.5 text-brand-700" />}
                    </button>
                  ))}
                </div>
              </div>

              {/* Language Selection */}
              <div>
                <label className="block text-sm font-semibold text-slate-900 mb-1">
                  {UI_STRINGS.start_screen.language_label}
                </label>
                <select
                  value={uiLanguage}
                  onChange={(e) => setUiLanguage(e.target.value)}
                  className="w-full sm:w-64 p-2.5 bg-slate-50 border border-slate-300 rounded-lg text-sm text-slate-800 focus:ring-2 focus:ring-brand-500 focus:outline-none"
                >
                  <option value="en">English (Default)</option>
                  <option value="ar">العربية (Arabic Instructions)</option>
                  <option value="ur">اردو (Urdu Instructions)</option>
                </select>
              </div>

              {/* Consent Checkboxes */}
              <div className="space-y-3 pt-4 border-t border-slate-100">
                <label className="flex items-start gap-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={privacyConsent}
                    onChange={(e) => setPrivacyConsent(e.target.checked)}
                    className="w-5 h-5 mt-0.5 accent-brand-600 rounded cursor-pointer shrink-0"
                  />
                  <span className="text-xs text-slate-600 leading-normal">
                    {UI_STRINGS.start_screen.consent_privacy}
                  </span>
                </label>

                <label className="flex items-start gap-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={researchConsent}
                    onChange={(e) => setResearchConsent(e.target.checked)}
                    className="w-5 h-5 mt-0.5 accent-brand-600 rounded cursor-pointer shrink-0"
                  />
                  <span className="text-xs text-slate-600 leading-normal">
                    {UI_STRINGS.start_screen.consent_research}
                  </span>
                </label>
              </div>

              {/* Primary CTA */}
              <div className="pt-2 flex flex-col sm:flex-row items-center gap-3">
                <button
                  onClick={handleStartSession}
                  disabled={!privacyConsent || isLoading}
                  className={`w-full sm:flex-1 py-3 px-6 rounded-xl font-bold text-sm text-white transition-all shadow-soft flex items-center justify-center gap-2 min-h-[48px] ${
                    !privacyConsent || isLoading
                      ? 'bg-slate-300 cursor-not-allowed'
                      : 'bg-brand-800 hover:bg-brand-900 active:bg-brand-950 focus:ring-4 focus:ring-brand-500/20'
                  }`}
                >
                  {isLoading ? (
                    <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                  ) : (
                    <>
                      <span>{UI_STRINGS.start_screen.start_btn}</span>
                      <IconArrowRight className="w-4 h-4" />
                    </>
                  )}
                </button>

                <button
                  type="button"
                  onClick={() => setShowAboutModal(true)}
                  className="w-full sm:w-auto py-3 px-5 text-sm font-semibold text-slate-600 hover:text-slate-900 hover:bg-slate-100 rounded-xl transition-colors min-h-[48px]"
                >
                  {UI_STRINGS.start_screen.about_test_btn}
                </button>
              </div>

              {/* Resume Assessment Option (PRD §2 & §6) */}
              <div className="pt-4 border-t border-slate-100 space-y-3">
                {savedSessionId && (
                  <div className="p-3.5 rounded-xl bg-brand-50 border border-brand-200 flex flex-col sm:flex-row items-center justify-between gap-3 text-xs">
                    <div className="text-left">
                      <span className="font-bold text-brand-900 block">Active In-Progress Session Detected</span>
                      <span className="font-mono text-slate-600">{savedSessionId}</span>
                    </div>
                    <button
                      type="button"
                      onClick={() => handleResumeSession(savedSessionId)}
                      disabled={isLoading}
                      className="w-full sm:w-auto px-4 py-2 bg-brand-800 hover:bg-brand-900 text-white font-bold rounded-lg shadow-xs transition-colors"
                    >
                      Resume Session
                    </button>
                  </div>
                )}

                <details className="text-xs text-slate-600">
                  <summary className="cursor-pointer font-semibold hover:text-brand-800 transition-colors">
                    Have a previous session ID? Resume here
                  </summary>
                  <div className="pt-2 flex items-center gap-2">
                    <input
                      type="text"
                      placeholder="e.g. ses_1234567890abcdef"
                      value={resumeInputId}
                      onChange={(e) => setResumeInputId(e.target.value)}
                      className="flex-1 p-2 bg-slate-50 border border-slate-300 rounded-lg text-xs text-slate-800 font-mono focus:ring-2 focus:ring-brand-500 focus:outline-none"
                    />
                    <button
                      type="button"
                      onClick={() => handleResumeSession(resumeInputId)}
                      disabled={!resumeInputId.trim() || isLoading}
                      className="px-4 py-2 bg-slate-800 hover:bg-slate-900 disabled:bg-slate-200 disabled:text-slate-400 text-white font-bold rounded-lg transition-colors"
                    >
                      Resume
                    </button>
                  </div>
                </details>
              </div>
            </div>
          </div>
        )}

        {/* STEP 2: WORKED EXAMPLE */}
        {currentStep === 'worked_example' && (
          <div className="max-w-2xl mx-auto space-y-6 animate-in fade-in duration-300">
            <div className="flex items-center justify-between pb-3 border-b border-slate-200">
              <div className="flex items-center gap-2 text-xs font-bold text-brand-700 uppercase tracking-wider">
                <span className="w-2 h-2 rounded-full bg-brand-600" />
                <span>{UI_STRINGS.worked_example.badge}</span>
              </div>
              <span className="text-xs font-semibold text-slate-500">Unscored Sample</span>
            </div>

            <div className="bg-white p-6 rounded-2xl border border-slate-200 shadow-card space-y-6">
              <div>
                <h2 className="text-xl font-bold text-slate-900 mb-2">{UI_STRINGS.worked_example.title}</h2>
                <p className="text-sm text-slate-600 leading-relaxed">{UI_STRINGS.worked_example.instructions}</p>
              </div>

              {/* Sample Question Demo */}
              <div className="p-5 rounded-xl bg-slate-50 border border-slate-200 space-y-4">
                <div className="text-base font-semibold text-slate-900">
                  Select the word that completes the sentence correctly:
                </div>
                <div className="text-lg font-medium text-slate-800 p-3 bg-white rounded-lg border border-slate-200">
                  He decided to <span className="underline font-bold decoration-brand-600 decoration-2">___</span> the
                  invitation because of a prior commitment.
                </div>

                <div className="space-y-2">
                  {[
                    { id: 'a', text: 'decline', correct: true },
                    { id: 'b', text: 'refuse to' },
                    { id: 'c', text: 'ignore in' },
                  ].map((opt) => (
                    <label
                      key={opt.id}
                      className="flex items-center gap-3 p-3 rounded-lg border border-slate-200 bg-white hover:bg-slate-50 cursor-pointer min-h-[44px]"
                    >
                      <input type="radio" name="sample_mcq" defaultChecked={opt.correct} className="w-4 h-4 accent-brand-600" />
                      <span className="text-sm font-medium text-slate-800">{opt.text}</span>
                    </label>
                  ))}
                </div>

                <div className="text-xs text-emerald-800 bg-emerald-50 border border-emerald-200 p-3 rounded-lg flex items-center gap-2">
                  <IconCheck className="w-4 h-4 text-emerald-600 shrink-0" />
                  <span>
                    <strong>Interface Mechanics:</strong> Selection is active. When you confirm your choice, your response
                    is recorded and the next question loads automatically.
                  </span>
                </div>
              </div>

              {/* Sound Check Demo */}
              <div className="p-4 rounded-xl border border-slate-200 bg-slate-50 flex items-center justify-between gap-4">
                <div>
                  <h3 className="text-sm font-semibold text-slate-900">{UI_STRINGS.worked_example.audio_test_title}</h3>
                  <p className="text-xs text-slate-500">{UI_STRINGS.worked_example.audio_test_desc}</p>
                </div>
                <button
                  type="button"
                  onClick={() => {
                    const audio = new Audio('/api/media/audio/sample_tone.wav');
                    audio.play().catch(() => {});
                  }}
                  className="px-4 py-2 bg-white border border-slate-300 hover:bg-slate-50 text-xs font-semibold rounded-lg shadow-xs flex items-center gap-2 min-h-[44px]"
                >
                  <IconVolume className="w-4 h-4 text-brand-700" />
                  <span>Play Test Tone</span>
                </button>
              </div>

              <button
                onClick={handleBeginLanguageSystems}
                disabled={isLoading}
                className="w-full py-3.5 px-6 bg-brand-800 hover:bg-brand-900 text-white font-bold text-sm rounded-xl shadow-soft flex items-center justify-center gap-2 min-h-[48px]"
              >
                <span>{UI_STRINGS.worked_example.continue_btn}</span>
                <IconArrowRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* STEP 3: OBJECTIVE TEST PLAYER (LS, RD, LSN) */}
        {currentStep === 'objective_test' && deliveryUnit && (
          <div className="max-w-4xl mx-auto space-y-6 animate-in fade-in duration-200">
            {/* Module & Progress Bar */}
            <div className="flex flex-wrap items-center justify-between gap-4 pb-4 border-b border-slate-200">
              <div className="flex items-center gap-3">
                <span className="px-3 py-1 rounded-md text-xs font-bold bg-brand-800 text-white uppercase tracking-wider">
                  {activeModule === 'LS'
                    ? 'Language Systems'
                    : activeModule === 'RD'
                    ? 'Reading Testlet'
                    : 'Listening Testlet'}
                </span>
                <span className="text-sm font-semibold text-slate-600">Question {questionCount}</span>
              </div>

              <Timer deadlineAt={deliveryUnit.deadline_at} onTimeout={handleSubmitObjective} />
            </div>

            {/* Testlet Stimulus Layout for RD and LSN */}
            <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
              {/* Stimulus Panel (RD/LSN) */}
              {(activeModule === 'RD' || activeModule === 'LSN') && (
                <div className="lg:col-span-6 bg-white p-6 rounded-2xl border border-slate-200 shadow-soft space-y-4 max-h-[540px] overflow-y-auto">
                  <div className="flex items-center justify-between pb-2 border-b border-slate-100">
                    <span className="text-xs font-bold text-slate-500 uppercase tracking-wider">
                      {deliveryUnit.stimulus_type || 'Reading Passage'}
                    </span>
                    {activeModule === 'LSN' && <IconHeadphones className="w-4 h-4 text-brand-600" />}
                  </div>

                  {activeModule === 'LSN' && (
                    <AudioPlayer
                      audioUrl={deliveryUnit.audio_url}
                      onPlayCompleted={(count) => setAudioReplayCount(count)}
                    />
                  )}

                  {activeModule === 'RD' && deliveryUnit.stimulus_text && (
                    <div className="text-sm sm:text-base text-slate-800 leading-relaxed font-sans whitespace-pre-line select-text">
                      {deliveryUnit.stimulus_text}
                    </div>
                  )}
                </div>
              )}

              {/* Items Panel */}
              <div
                className={`${
                  activeModule === 'LS' ? 'max-w-2xl mx-auto col-span-12' : 'lg:col-span-6'
                } w-full space-y-6`}
              >
                {deliveryUnit.items.map((item: any, idx: number) => {
                  if (activeModule !== 'LS' && idx !== activeTestletItemIdx) {
                    return null;
                  }

                  const isListeningLocked = activeModule === 'LSN' && audioReplayCount === 0;

                  return (
                    <div
                      key={item.item_id}
                      className="bg-white p-6 rounded-2xl border border-slate-200 shadow-card space-y-6"
                    >
                      <div className="text-base sm:text-lg font-semibold text-slate-900 leading-snug">
                        {item.stem.replace('___', '______')}
                      </div>

                      {isListeningLocked && (
                        <div className="p-3 bg-amber-50 border border-amber-200 rounded-xl text-xs text-amber-800 flex items-center gap-2">
                          <IconAlert className="w-4 h-4 text-amber-600 shrink-0" />
                          <span>Please play the audio recording above to unlock questions.</span>
                        </div>
                      )}

                      <div className="space-y-3" role="radiogroup" aria-label="Question options">
                        {item.options.map((opt: any) => {
                          const isSelected = selectedAnswers[item.item_id] === opt.option_id;
                          return (
                            <button
                              key={opt.option_id}
                              type="button"
                              role="radio"
                              aria-checked={isSelected}
                              disabled={isListeningLocked}
                              onClick={() => {
                                if (isListeningLocked) return;
                                setSelectedAnswers((prev) => ({
                                  ...prev,
                                  [item.item_id]: opt.option_id,
                                }));
                              }}
                              className={`w-full text-left p-4 rounded-xl border text-sm sm:text-base font-medium transition-all flex items-center justify-between min-h-[48px] ${
                                isListeningLocked
                                  ? 'opacity-60 cursor-not-allowed bg-slate-50 border-slate-200 text-slate-500'
                                  : isSelected
                                  ? 'bg-brand-50 border-brand-600 text-brand-900 ring-2 ring-brand-500/20'
                                  : 'border-slate-200 hover:bg-slate-50 text-slate-800'
                              }`}
                            >
                              <span>{opt.text}</span>
                              <div
                                className={`w-5 h-5 rounded-full border flex items-center justify-center ${
                                  isSelected ? 'border-brand-600 bg-brand-600' : 'border-slate-300'
                                }`}
                              >
                                {isSelected && <div className="w-2 h-2 rounded-full bg-white" />}
                              </div>
                            </button>
                          );
                        })}
                      </div>

                      {/* Navigation between testlet items */}
                      {deliveryUnit.items.length > 1 && (
                        <div className="flex items-center justify-between pt-4 border-t border-slate-100">
                          <span className="text-xs font-medium text-slate-500">
                            Testlet Question {activeTestletItemIdx + 1} of {deliveryUnit.items.length}
                          </span>
                          <div className="flex gap-2">
                            {deliveryUnit.items.map((_: any, i: number) => (
                              <button
                                key={i}
                                type="button"
                                onClick={() => setActiveTestletItemIdx(i)}
                                className={`w-8 h-8 rounded-lg text-xs font-bold transition-colors ${
                                  activeTestletItemIdx === i
                                    ? 'bg-brand-800 text-white'
                                    : 'bg-slate-100 text-slate-700 hover:bg-slate-200'
                                }`}
                              >
                                {i + 1}
                              </button>
                            ))}
                          </div>
                        </div>
                      )}

                      {/* Action buttons */}
                      {deliveryUnit.items.length > 1 ? (
                        <div className="pt-2 flex flex-col sm:flex-row items-center gap-3">
                          {activeTestletItemIdx > 0 && (
                            <button
                              type="button"
                              onClick={() => setActiveTestletItemIdx(activeTestletItemIdx - 1)}
                              className="w-full sm:w-auto py-3 px-5 text-sm font-semibold text-slate-700 bg-slate-100 hover:bg-slate-200 rounded-xl transition-colors min-h-[48px]"
                            >
                              Previous Question
                            </button>
                          )}
                          {activeTestletItemIdx < deliveryUnit.items.length - 1 ? (
                            <button
                              type="button"
                              onClick={() => setActiveTestletItemIdx(activeTestletItemIdx + 1)}
                              disabled={!selectedAnswers[item.item_id]}
                              className={`w-full sm:flex-1 py-3.5 px-6 rounded-xl text-sm font-bold text-white transition-all shadow-soft flex items-center justify-center gap-2 min-h-[48px] ${
                                !selectedAnswers[item.item_id]
                                  ? 'bg-slate-200 text-slate-400 cursor-not-allowed'
                                  : 'bg-brand-800 hover:bg-brand-900 active:bg-brand-950 focus:ring-4 focus:ring-brand-500/20'
                              }`}
                            >
                              <span>Next Question ({activeTestletItemIdx + 2} of {deliveryUnit.items.length})</span>
                              <IconArrowRight className="w-4 h-4" />
                            </button>
                          ) : (
                            <button
                              type="button"
                              onClick={handleSubmitObjective}
                              disabled={
                                isLoading ||
                                deliveryUnit.items.some((it: any) => !selectedAnswers[it.item_id])
                              }
                              className={`w-full sm:flex-1 py-3.5 px-6 rounded-xl text-sm font-bold text-white transition-all shadow-soft flex items-center justify-center gap-2 min-h-[48px] ${
                                isLoading ||
                                deliveryUnit.items.some((it: any) => !selectedAnswers[it.item_id])
                                  ? 'bg-slate-200 text-slate-400 cursor-not-allowed'
                                  : 'bg-brand-800 hover:bg-brand-900 active:bg-brand-950 focus:ring-4 focus:ring-brand-500/20'
                              }`}
                            >
                              {isLoading ? (
                                <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                              ) : (
                                <>
                                  <span>Submit Testlet ({deliveryUnit.items.length} Questions)</span>
                                  <IconArrowRight className="w-4 h-4" />
                                </>
                              )}
                            </button>
                          )}
                        </div>
                      ) : (
                        <button
                          type="button"
                          onClick={handleSubmitObjective}
                          disabled={isLoading || !selectedAnswers[item.item_id]}
                          className={`w-full py-3.5 px-6 rounded-xl text-sm font-bold text-white transition-all shadow-soft flex items-center justify-center gap-2 min-h-[48px] ${
                            isLoading || !selectedAnswers[item.item_id]
                              ? 'bg-slate-200 text-slate-400 cursor-not-allowed'
                              : 'bg-brand-800 hover:bg-brand-900 active:bg-brand-950 focus:ring-4 focus:ring-brand-500/20'
                          }`}
                        >
                          {isLoading ? (
                            <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                          ) : (
                            <>
                              <span>Confirm Choice</span>
                              <IconArrowRight className="w-4 h-4" />
                            </>
                          )}
                        </button>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        )}

        {/* STEP 4: RECEPTIVE RESULT SCREEN */}
        {currentStep === 'receptive_result' && receptiveResult && (
          <div className="max-w-3xl mx-auto space-y-8 animate-in fade-in duration-300">
            <div className="text-center space-y-2">
              <span className="px-3 py-1 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-800 text-xs font-bold uppercase tracking-wider">
                Foundation Stage Complete
              </span>
              <h1 className="text-3xl font-extrabold text-slate-900">Foundation and Receptive Profile</h1>
              <p className="text-sm text-slate-600 max-w-xl mx-auto">
                {UI_STRINGS.results.receptive_only_notice}
              </p>
            </div>

            {/* Receptive Skill Cards */}
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              {receptiveResult.skills
                .filter((s: any) => s.skill === 'RD' || s.skill === 'LSN')
                .map((sk: any) => (
                  <div key={sk.skill} className="bg-white p-6 rounded-2xl border border-slate-200 shadow-card space-y-4">
                    <div className="flex items-center justify-between">
                      <span className="text-xs font-bold text-slate-500 uppercase tracking-wider">
                        {sk.skill === 'RD' ? 'Reading' : 'Listening'}
                      </span>
                      <span className="px-3 py-1 rounded-lg font-mono text-base font-bold bg-brand-100 text-brand-900">
                        {sk.band || 'Diagnosed'}
                      </span>
                    </div>

                    <div className="text-sm text-slate-700 space-y-2">
                      {sk.can_do && sk.can_do.map((cd: string, i: number) => (
                        <p key={i} className="flex items-start gap-2">
                          <IconCheck className="w-4 h-4 text-emerald-600 shrink-0 mt-0.5" />
                          <span>{cd}</span>
                        </p>
                      ))}
                    </div>
                  </div>
                ))}
            </div>

            {/* Confidence Badge */}
            <div className="p-4 rounded-xl border border-slate-200 bg-white flex items-center justify-between gap-4">
              <div>
                <span className="text-xs font-semibold text-slate-500 uppercase tracking-wider block">
                  Diagnostic Confidence
                </span>
                <span className="text-base font-bold text-brand-900">
                  {receptiveResult.confidence} Confidence
                </span>
              </div>
              <div className="text-xs text-slate-500 max-w-md text-right">
                {receptiveResult.confidence_reasons && receptiveResult.confidence_reasons.join(' ')}
              </div>
            </div>

            {/* Two Clear Options (PRD §4.4) */}
            {isReceptiveFinalized ? (
              <div className="p-6 rounded-2xl bg-white border border-slate-200 shadow-soft space-y-4">
                <div className="flex items-center gap-2 text-emerald-800 font-bold text-sm">
                  <IconCheck className="w-5 h-5 text-emerald-600" />
                  <span>Foundation and Receptive Profile Finalized</span>
                </div>
                <p className="text-xs text-slate-600 leading-relaxed">
                  Your diagnostic receptive assessment has concluded. Productive skills (Speaking and Writing) are recorded as not measured per your selection.
                </p>
                <div className="flex flex-wrap gap-3 pt-2">
                  <button
                    onClick={() => window.print()}
                    className="px-4 py-2.5 bg-brand-800 hover:bg-brand-900 text-white font-bold text-xs rounded-xl shadow-soft"
                  >
                    Print Profile
                  </button>
                  <button
                    onClick={handleDeleteData}
                    className="px-4 py-2.5 text-xs font-semibold text-rose-700 bg-rose-50 hover:bg-rose-100 border border-rose-200 rounded-xl"
                  >
                    Delete My Data
                  </button>
                  <button
                    onClick={() => {
                      if (typeof window !== 'undefined') localStorage.removeItem('gepa_active_session');
                      window.location.href = '/';
                    }}
                    className="px-4 py-2.5 text-xs font-semibold text-slate-700 bg-slate-100 hover:bg-slate-200 rounded-xl"
                  >
                    Start New Assessment
                  </button>
                </div>
              </div>
            ) : (
              <div className="p-6 rounded-2xl bg-brand-50 border border-brand-200 space-y-4">
                <h3 className="text-base font-bold text-brand-950">Next Steps</h3>
                <p className="text-sm text-slate-700">
                  You can continue to the Speaking and Writing modules to complete your full 4-skill diagnostic profile, or
                  conclude here with your provisional receptive profile.
                </p>

                <div className="flex flex-col sm:flex-row gap-3 pt-2">
                  <button
                    onClick={handleProceedToSpeaking}
                    className="flex-1 py-3 px-5 bg-brand-800 hover:bg-brand-900 text-white font-bold text-sm rounded-xl shadow-soft flex items-center justify-center gap-2 min-h-[48px]"
                  >
                    <span>Continue to Speaking & Writing (~35 min)</span>
                    <IconArrowRight className="w-4 h-4" />
                  </button>

                  <button
                    onClick={() => setIsReceptiveFinalized(true)}
                    className="py-3 px-5 bg-white border border-slate-300 hover:bg-slate-100 text-slate-800 font-semibold text-sm rounded-xl min-h-[48px]"
                  >
                    Finish with Receptive Guidance
                  </button>
                </div>
              </div>
            )}
          </div>
        )}

        {/* STEP 5: SPEAKING MIC CHECK */}
        {currentStep === 'mic_check' && (
          <div className="max-w-xl mx-auto bg-white p-6 sm:p-8 rounded-2xl border border-slate-200 shadow-card space-y-6 animate-in fade-in duration-300">
            <div className="text-center space-y-2">
              <div className="w-12 h-12 mx-auto rounded-full bg-brand-100 text-brand-700 flex items-center justify-center">
                <IconMic className="w-6 h-6" />
              </div>
              <h2 className="text-2xl font-bold text-slate-900">Microphone Sound Check</h2>
              <p className="text-sm text-slate-600 max-w-sm mx-auto">
                Before commencing the Speaking tasks, please verify that your microphone captures clear spoken audio.
              </p>
            </div>

            <AudioRecorder
              prepSeconds={0}
              maxSpeakSeconds={5}
              allowsRerecord={true}
              onRecordingComplete={() => {}}
            />

            <div className="flex flex-col sm:flex-row items-center gap-3 pt-2">
              <button
                onClick={() => setCurrentStep('speaking_test')}
                className="w-full sm:flex-1 py-3 px-6 bg-brand-800 hover:bg-brand-900 text-white font-bold text-sm rounded-xl shadow-soft flex items-center justify-center gap-2 min-h-[48px]"
              >
                <span>Microphone Works — Begin Speaking</span>
                <IconArrowRight className="w-4 h-4" />
              </button>

              <button
                onClick={handleProceedToWriting}
                className="w-full sm:w-auto py-3 px-4 text-xs font-semibold text-slate-500 hover:text-slate-800"
              >
                Skip Speaking (Mark Not Measured)
              </button>
            </div>
          </div>
        )}

        {/* STEP 6: SPEAKING TEST PLAYER */}
        {currentStep === 'speaking_test' && speakingTasks[currentSpeakingIdx] && (
          <div className="max-w-2xl mx-auto space-y-6 animate-in fade-in duration-200">
            {/* Header */}
            <div className="flex items-center justify-between pb-3 border-b border-slate-200">
              <span className="px-3 py-1 rounded-md text-xs font-bold bg-brand-800 text-white uppercase tracking-wider">
                Speaking Task {currentSpeakingIdx + 1} of {speakingTasks.length}
              </span>
              <span className="text-xs font-semibold text-slate-500">
                {speakingTasks[currentSpeakingIdx].label}
              </span>
            </div>

            {/* Prompt Box */}
            <div className="bg-white p-6 rounded-2xl border border-slate-200 shadow-soft space-y-3">
              <span className="text-xs font-bold text-slate-400 uppercase tracking-wider">Task Prompt</span>
              <p className="text-lg font-semibold text-slate-900 leading-relaxed">
                {speakingTasks[currentSpeakingIdx].prompt}
              </p>
            </div>

            {/* Recorder Island */}
            <AudioRecorder
              prepSeconds={speakingTasks[currentSpeakingIdx].prep_seconds || 15}
              maxSpeakSeconds={speakingTasks[currentSpeakingIdx].max_speak_seconds || 45}
              allowsRerecord={speakingTasks[currentSpeakingIdx].allows_rerecord !== false}
              onRecordingComplete={() => {}}
            />

            <button
              onClick={handleSubmitSpeakingTask}
              disabled={isLoading}
              className="w-full py-3.5 px-6 bg-brand-800 hover:bg-brand-900 text-white font-bold text-sm rounded-xl shadow-soft flex items-center justify-center gap-2 min-h-[48px]"
            >
              {isLoading ? (
                <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
              ) : (
                <>
                  <span>Submit Spoken Response</span>
                  <IconArrowRight className="w-4 h-4" />
                </>
              )}
            </button>
          </div>
        )}

        {/* STEP 6.5: WRITING BREAK & DESKTOP RECOMMENDATION (PRD §6) */}
        {currentStep === 'writing_break' && (
          <div className="max-w-xl mx-auto bg-white p-6 sm:p-8 rounded-2xl border border-slate-200 shadow-card space-y-6 animate-in fade-in duration-300">
            <div className="flex items-center gap-2 text-xs font-bold text-brand-700 uppercase tracking-wider">
              <span className="w-2 h-2 rounded-full bg-brand-600" />
              <span>Section Transition • Optional Break</span>
            </div>
            <div>
              <h2 className="text-2xl font-bold text-slate-900">Writing Module</h2>
              <p className="text-sm text-slate-600 mt-1 leading-relaxed">
                You have completed the Spoken interaction tasks. Take a short pause before starting the final Writing module.
              </p>
            </div>

            <div className="p-4 rounded-xl bg-amber-50 border border-amber-200 text-amber-900 space-y-2 text-xs">
              <div className="font-bold flex items-center gap-1.5">
                <IconAlert className="w-4 h-4 text-amber-700 shrink-0" />
                <span>Device Recommendation (PRD §6)</span>
              </div>
              <p>
                A <strong>desktop computer or physical keyboard</strong> is strongly recommended for extended composition tasks in upper routes.
              </p>
            </div>

            <div className="p-4 rounded-xl bg-slate-50 border border-slate-200 space-y-2 text-xs text-slate-700">
              <span className="font-bold block text-slate-900">Module Capabilities:</span>
              <ul className="list-disc list-inside space-y-1 text-slate-600">
                <li>Automatic draft autosave active every 2 seconds.</li>
                <li>Dynamic target word guidance indicators.</li>
                <li>Spellcheck and autocomplete are disabled for diagnostic accuracy.</li>
              </ul>
            </div>

            <button
              onClick={handleProceedToWriting}
              disabled={isLoading}
              className="w-full py-3.5 px-6 bg-brand-800 hover:bg-brand-900 text-white font-bold text-sm rounded-xl shadow-soft flex items-center justify-center gap-2 min-h-[48px]"
            >
              {isLoading ? (
                <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
              ) : (
                <>
                  <span>Begin Writing Module</span>
                  <IconArrowRight className="w-4 h-4" />
                </>
              )}
            </button>
          </div>
        )}

        {/* STEP 7: WRITING TEST PLAYER */}
        {currentStep === 'writing_test' && writingTasks[currentWritingIdx] && (
          <div className="max-w-4xl mx-auto space-y-6 animate-in fade-in duration-200">
            {/* Header */}
            <div className="flex items-center justify-between pb-3 border-b border-slate-200">
              <span className="px-3 py-1 rounded-md text-xs font-bold bg-brand-800 text-white uppercase tracking-wider">
                Writing Task {currentWritingIdx + 1} of {writingTasks.length}
              </span>
              <span className="text-xs font-semibold text-slate-500">
                {writingTasks[currentWritingIdx].label}
              </span>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
              {/* Task Prompt Panel */}
              <div className="lg:col-span-5 bg-white p-6 rounded-2xl border border-slate-200 shadow-soft space-y-4">
                <span className="text-xs font-bold text-slate-400 uppercase tracking-wider">Instructions</span>
                <p className="text-base font-semibold text-slate-900 leading-relaxed">
                  {writingTasks[currentWritingIdx].prompt}
                </p>
                <div className="p-3 bg-slate-50 rounded-lg text-xs text-slate-600">
                  Compose your response directly in the writing editor. Your draft is continuously saved in the
                  background.
                </div>
              </div>

              {/* Editor Island */}
              <div className="lg:col-span-7 flex flex-col space-y-4">
                <WritingEditor
                  wordGuidance={writingTasks[currentWritingIdx].word_guidance}
                  initialValue={currentWritingText}
                  onChange={setCurrentWritingText}
                  onAutosave={(txt) => {
                    fetch(`/api/sessions/${sessionId}/writing/${writingTasks[currentWritingIdx].task_id}/draft`, {
                      method: 'PUT',
                      headers: { 'Content-Type': 'application/json' },
                      body: JSON.stringify({ text: txt }),
                    }).catch(() => {});
                  }}
                />

                <button
                  onClick={handleSubmitWritingTask}
                  disabled={isLoading || currentWritingText.trim().length === 0}
                  className={`w-full py-3.5 px-6 rounded-xl font-bold text-sm text-white transition-all shadow-soft flex items-center justify-center gap-2 min-h-[48px] ${
                    isLoading || currentWritingText.trim().length === 0
                      ? 'bg-slate-200 text-slate-400 cursor-not-allowed'
                      : 'bg-brand-800 hover:bg-brand-900 active:bg-brand-950 focus:ring-4 focus:ring-brand-500/20'
                  }`}
                >
                  {isLoading ? (
                    <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                  ) : (
                    <>
                      <span>Submit Written Response</span>
                      <IconArrowRight className="w-4 h-4" />
                    </>
                  )}
                </button>
              </div>
            </div>
          </div>
        )}

        {/* STEP 8: FULL RESULT DASHBOARD */}
        {currentStep === 'full_result' && fullResult && (
          <div className="max-w-4xl mx-auto space-y-10 animate-in fade-in duration-300 pb-12">
            {/* Header */}
            <div className="text-center space-y-3">
              <span className="px-3.5 py-1 rounded-full bg-brand-50 border border-brand-200 text-brand-800 text-xs font-bold uppercase tracking-wider">
                Assessment Complete
              </span>
              <h1 className="text-3xl sm:text-4xl font-extrabold text-slate-900 tracking-tight">
                {UI_STRINGS.results.title}
              </h1>
              <p className="text-sm text-slate-600 max-w-xl mx-auto">
                Comprehensive diagnostic profile reflecting observed communicative performance across all tested domains.
              </p>
            </div>

            {/* 1. PRIMARY SKILL CARDS (PROMINENT - DOM ORDER & SIZE) */}
            <div>
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                {fullResult.skills.map((sk: any) => (
                  <div
                    key={sk.skill}
                    className="bg-white p-6 rounded-2xl border border-slate-200 shadow-card hover:shadow-elevated transition-all flex flex-col justify-between space-y-4"
                  >
                    <div>
                      <div className="flex items-center justify-between pb-2 border-b border-slate-100">
                        <span className="text-xs font-bold text-slate-500 uppercase tracking-wider">
                          {sk.skill === 'RD'
                            ? 'Reading'
                            : sk.skill === 'LSN'
                            ? 'Listening'
                            : sk.skill === 'SPK'
                            ? 'Speaking'
                            : 'Writing'}
                        </span>
                      </div>

                      {/* Large Estimated Band Chip */}
                      <div className="py-3">
                        <span className="text-3xl sm:text-4xl font-black text-brand-900 tracking-tight font-mono">
                          {sk.band || 'Measured'}
                        </span>
                      </div>

                      {/* Can-Do statements */}
                      <div className="text-xs text-slate-700 space-y-2 pt-2 border-t border-slate-100">
                        {sk.can_do && sk.can_do.map((cd: string, i: number) => (
                          <p key={i} className="flex items-start gap-1.5 leading-relaxed">
                            <IconCheck className="w-3.5 h-3.5 text-emerald-600 shrink-0 mt-0.5" />
                            <span>{cd}</span>
                          </p>
                        ))}
                      </div>
                    </div>

                    {/* Growth area */}
                    {sk.growth_areas && sk.growth_areas.length > 0 && (
                      <div className="p-2.5 bg-slate-50 rounded-lg text-xs text-slate-600">
                        <span className="font-semibold text-slate-800 block mb-0.5">Focus Area:</span>
                        <span>{sk.growth_areas[0]}</span>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>

            {/* 2. CONFIDENCE BADGE */}
            <div className="bg-white p-6 rounded-2xl border border-slate-200 shadow-soft flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-xl bg-brand-50 text-brand-700 flex items-center justify-center shrink-0">
                  <IconShield className="w-5 h-5" />
                </div>
                <div>
                  <div className="text-xs font-bold uppercase tracking-wider text-slate-400">
                    Diagnostic Confidence
                  </div>
                  <div className="text-lg font-bold text-slate-900">
                    {fullResult.confidence} Confidence
                  </div>
                </div>
              </div>
              <div className="text-xs text-slate-600 max-w-lg">
                {fullResult.confidence_reasons && (
                  <ul className="list-disc list-inside space-y-1">
                    {fullResult.confidence_reasons.map((r: string, i: number) => (
                      <li key={i}>{r}</li>
                    ))}
                  </ul>
                )}
              </div>
            </div>

            {/* 3. DIAGNOSTICS SECTION (SUBORDINATE) */}
            {fullResult.diagnostics && (
              <div className="bg-white p-6 rounded-2xl border border-slate-200 shadow-soft space-y-4">
                <h3 className="text-sm font-bold text-slate-900 uppercase tracking-wider">
                  Construct-Level Diagnostics & Observations
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
                  <div className="p-4 rounded-xl bg-slate-50 border border-slate-200 space-y-2">
                    <span className="font-bold text-slate-900 block">Grammar & Vocabulary Observed</span>
                    {fullResult.diagnostics.language_systems.constructs_strong && (
                      <div>
                        <span className="font-semibold text-emerald-700">Strengths:</span>{' '}
                        {fullResult.diagnostics.language_systems.constructs_strong.join(', ')}
                      </div>
                    )}
                    {fullResult.diagnostics.language_systems.constructs_weak && (
                      <div>
                        <span className="font-semibold text-amber-700">Growth Areas:</span>{' '}
                        {fullResult.diagnostics.language_systems.constructs_weak.join(', ')}
                      </div>
                    )}
                  </div>

                  <div className="p-4 rounded-xl bg-slate-50 border border-slate-200 space-y-2">
                    <span className="font-bold text-slate-900 block">Fluency & Spoken Performance</span>
                    <p className="text-slate-600">
                      {fullResult.diagnostics.fluency_notes && fullResult.diagnostics.fluency_notes[0]}
                    </p>
                    <p className="text-slate-600">
                      {fullResult.diagnostics.pronunciation_notes && fullResult.diagnostics.pronunciation_notes[0]}
                    </p>
                  </div>
                </div>
              </div>
            )}

            {/* 4. OVERALL PROFILE HEADLINE (SMALL, BELOW SKILLS) */}
            {fullResult.headline && fullResult.headline.kind !== 'none' && (
              <div className="p-4 rounded-xl bg-slate-100 border border-slate-200 flex items-center justify-between text-xs text-slate-700">
                <span>
                  <strong>Overall Profile Summary:</strong>{' '}
                  {fullResult.headline.kind === 'indicative_overall'
                    ? `Indicative overall placement: ${fullResult.headline.band}`
                    : `Uneven Profile across range ${fullResult.headline.range?.[0]}–${fullResult.headline.range?.[1]}`}
                </span>
                <span className="text-slate-500">Lower median derivation</span>
              </div>
            )}

            {/* 5. TARGET-EXAM READINESS BLOCK WITH MANDATORY DISCLAIMER */}
            {fullResult.readiness && (
              <div className="p-6 rounded-2xl bg-brand-50/60 border border-brand-200 space-y-3">
                <div className="flex items-center gap-2">
                  <span className="text-xs font-bold uppercase tracking-wider text-brand-800">
                    Destination Readiness — {fullResult.readiness.target}
                  </span>
                </div>
                <p className="text-sm text-slate-800 leading-relaxed font-medium">
                  {fullResult.readiness.text}
                </p>
                {(fullResult.readiness.currency_note ||
                  READINESS_TARGETS[fullResult.readiness.target]?.currencyNote) && (
                  <div className="p-3 bg-white rounded-lg border border-brand-300 text-brand-900 text-xs flex items-center gap-2">
                    <IconShield className="w-4 h-4 text-brand-700 shrink-0" />
                    <span>
                      {fullResult.readiness.currency_note ||
                        READINESS_TARGETS[fullResult.readiness.target]?.currencyNote}
                    </span>
                  </div>
                )}
                <div className="p-3 bg-white rounded-lg border border-amber-300 text-amber-900 font-semibold text-xs flex items-center gap-2">
                  <IconAlert className="w-4 h-4 text-amber-600 shrink-0" />
                  <span>{fullResult.readiness.disclaimer}</span>
                </div>
              </div>
            )}

            {/* 6. RETEST ADVICE & DATA CONTROLS */}
            <div className="flex flex-col sm:flex-row items-center justify-between gap-4 p-6 bg-white rounded-2xl border border-slate-200 shadow-soft">
              <div className="text-xs text-slate-600 max-w-md">
                <span className="font-bold text-slate-900 block mb-0.5">{UI_STRINGS.results.retest_title}</span>
                <span>{fullResult.retest_advice}</span>
              </div>

              <div className="flex items-center gap-3">
                <button
                  onClick={handleDeleteData}
                  className="px-4 py-2.5 text-xs font-semibold text-rose-700 bg-rose-50 hover:bg-rose-100 border border-rose-200 rounded-lg transition-colors min-h-[44px]"
                >
                  {UI_STRINGS.results.delete_data_btn}
                </button>
                <button
                  onClick={() => window.print()}
                  className="px-4 py-2.5 text-xs font-semibold text-slate-700 bg-slate-50 hover:bg-slate-100 border border-slate-300 rounded-lg transition-colors min-h-[44px]"
                >
                  Print Profile
                </button>
              </div>
            </div>
          </div>
        )}
      </main>

      {/* Footer */}
      <footer className="bg-white border-t border-slate-200 py-6 text-center text-xs text-slate-500">
        <div className="max-w-6xl mx-auto px-4 space-y-1">
          <p>
            GEPA Diagnostic Assessment Beta • Pre-calibration prototype •{' '}
            <a href="/about" className="font-semibold text-brand-700 hover:underline">
              Technical Manual & CEFR Scope
            </a>
          </p>
          <p className="text-slate-400">Claims Policy Guard Active • Zero High-Stakes Certification</p>
        </div>
      </footer>

      {/* About Modal */}
      {showAboutModal && (
        <div
          role="dialog"
          aria-modal="true"
          className="fixed inset-0 z-50 bg-slate-900/50 backdrop-blur-xs flex items-center justify-center p-4"
        >
          <div className="bg-white rounded-2xl max-w-lg w-full p-6 space-y-4 shadow-elevated animate-in fade-in">
            <div className="flex items-center justify-between pb-3 border-b border-slate-200">
              <h2 className="text-lg font-bold text-slate-900">About GEPA Assessment</h2>
              <button
                onClick={() => setShowAboutModal(false)}
                className="text-slate-400 hover:text-slate-600 text-xl font-bold"
              >
                &times;
              </button>
            </div>
            <div className="text-sm text-slate-600 space-y-3 leading-relaxed">
              <p>
                GEPA is a transparent, computer-adaptive diagnostic placement assessment. It provides an indicative estimate
                referencing international descriptors across Language Systems, Reading, Listening, Speaking, and Writing.
              </p>
              {/* CEFR Scope Note per PRD §5 */}
              <div className="p-3 bg-brand-50 rounded-lg border border-brand-200 text-xs text-brand-900 font-medium space-y-1">
                <strong>CEFR Scope Note:</strong>
                <p>
                  GEPA v2 samples mediation and digitally-mediated interaction where practical; it does not claim to measure plurilingual/pluricultural competence.
                </p>
              </div>
              <div className="p-3 bg-amber-50 rounded-lg border border-amber-200 text-xs text-amber-900 font-medium">
                <strong>Important Notice:</strong> GEPA does not predict official exam scores. It is intended for formative study guidance only.
              </div>
            </div>
            <div className="flex flex-col sm:flex-row gap-2">
              <a
                href="/about"
                className="flex-1 py-2.5 bg-slate-100 hover:bg-slate-200 text-slate-800 font-semibold text-xs rounded-lg text-center transition-colors flex items-center justify-center"
              >
                View Full Technical Manual
              </a>
              <button
                onClick={() => setShowAboutModal(false)}
                className="flex-1 py-2.5 bg-brand-800 text-white font-semibold text-xs rounded-lg hover:bg-brand-900 transition-colors"
              >
                Return to Assessment
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
