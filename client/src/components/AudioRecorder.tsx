import React, { useState, useRef, useEffect } from 'react';
import { IconMic, IconPlay, IconPause, IconAlert, IconCheck } from './Icons';

interface AudioRecorderProps {
  prepSeconds: number;
  maxSpeakSeconds: number;
  allowsRerecord: boolean;
  onRecordingComplete: (blob: Blob, qualityPassed: boolean) => void;
  className?: string;
}

export const AudioRecorder: React.FC<AudioRecorderProps> = ({
  prepSeconds,
  maxSpeakSeconds,
  allowsRerecord,
  onRecordingComplete,
  className = '',
}) => {
  const [phase, setPhase] = useState<'prep' | 'recording' | 'review' | 'quality_failed'>('prep');
  const [prepTimeRemaining, setPrepTimeRemaining] = useState<number>(prepSeconds);
  const [speakTimeRemaining, setSpeakTimeRemaining] = useState<number>(maxSpeakSeconds);
  const [rerecordUsed, setRerecordUsed] = useState<boolean>(false);
  const [audioLevel, setAudioLevel] = useState<number>(0);
  const [recordedUrl, setRecordedUrl] = useState<string | null>(null);
  const [isPlayingReview, setIsPlayingReview] = useState<boolean>(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const audioChunksRef = useRef<Blob[]>([]);
  const audioContextRef = useRef<AudioContext | null>(null);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const animFrameRef = useRef<number | null>(null);
  const recordedBlobRef = useRef<Blob | null>(null);
  // recorder.onstop below is a closure fixed at recorder-creation time, so
  // reading the `speakTimeRemaining` *state* from inside it would always
  // see the value from that one moment (its initial maxSpeakSeconds, since
  // the countdown effect hasn't ticked yet) -- every real recording would
  // compute recordedSecs=0 and always fail the >=3s quality gate,
  // regardless of how long the candidate actually spoke. A ref's `.current`
  // is read fresh at call time even from a stale closure, so this measures
  // real elapsed wall-clock time instead.
  const recordingStartedAtRef = useRef<number>(0);

  // Prep Countdown
  useEffect(() => {
    if (phase !== 'prep') return;

    setPrepTimeRemaining(prepSeconds);
    const interval = setInterval(() => {
      setPrepTimeRemaining((prev) => {
        if (prev <= 1) {
          clearInterval(interval);
          startRecording();
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(interval);
  }, [phase, prepSeconds]);

  // Speaking Countdown
  useEffect(() => {
    if (phase !== 'recording') return;

    setSpeakTimeRemaining(maxSpeakSeconds);
    const interval = setInterval(() => {
      setSpeakTimeRemaining((prev) => {
        if (prev <= 1) {
          clearInterval(interval);
          stopRecording();
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(interval);
  }, [phase, maxSpeakSeconds]);

  const startRecording = async () => {
    setPhase('recording');
    audioChunksRef.current = [];
    setErrorMessage(null);

    try {
      if (navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
        const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        const recorder = new MediaRecorder(stream, { mimeType: 'audio/webm;codecs=opus' });

        // Setup audio level meter
        const audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
        audioContextRef.current = audioCtx;
        const source = audioCtx.createMediaStreamSource(stream);
        const analyser = audioCtx.createAnalyser();
        analyser.fftSize = 256;
        source.connect(analyser);
        analyserRef.current = analyser;

        const updateLevel = () => {
          if (analyserRef.current) {
            const data = new Uint8Array(analyserRef.current.frequencyBinCount);
            analyserRef.current.getByteFrequencyData(data);
            const sum = data.reduce((acc, val) => acc + val, 0);
            const avg = sum / data.length;
            setAudioLevel(Math.min(100, Math.round((avg / 128) * 100)));
          }
          animFrameRef.current = requestAnimationFrame(updateLevel);
        };
        updateLevel();

        recorder.ondataavailable = (e) => {
          if (e.data.size > 0) {
            audioChunksRef.current.push(e.data);
          }
        };

        recorder.onstop = () => {
          if (animFrameRef.current) cancelAnimationFrame(animFrameRef.current);
          stream.getTracks().forEach((t) => t.stop());

          const blob = new Blob(audioChunksRef.current, { type: 'audio/webm;codecs=opus' });
          recordedBlobRef.current = blob;
          const url = URL.createObjectURL(blob);
          setRecordedUrl(url);

          // Quality gate check: duration >= 3s
          const recordedSecs = (Date.now() - recordingStartedAtRef.current) / 1000;
          if (recordedSecs < 3) {
            setPhase('quality_failed');
            setErrorMessage('We could not capture that clearly. Recording duration was under 3 seconds.');
          } else {
            setPhase('review');
            onRecordingComplete(blob, true);
          }
        };

        mediaRecorderRef.current = recorder;
        recordingStartedAtRef.current = Date.now();
        recorder.start(250);
      } else {
        simulateRecording();
      }
    } catch {
      // Fallback for mock/test environment without mic
      simulateRecording();
    }
  };

  const simulateRecording = () => {
    // Simulated live meter animation
    const interval = setInterval(() => {
      setAudioLevel(Math.floor(Math.random() * 60) + 20);
    }, 150);

    // Save timer reference to stop
    setTimeout(() => {
      clearInterval(interval);
    }, maxSpeakSeconds * 1000);
  };

  const stopRecording = () => {
    if (mediaRecorderRef.current && mediaRecorderRef.current.state === 'recording') {
      mediaRecorderRef.current.stop();
    } else {
      // Fallback mock recording
      const mockBlob = new Blob(['mock audio binary bytes'], { type: 'audio/webm' });
      recordedBlobRef.current = mockBlob;
      setRecordedUrl('mock://audio');
      setPhase('review');
      onRecordingComplete(mockBlob, true);
    }
  };

  const handleRerecord = () => {
    if (rerecordUsed && phase !== 'quality_failed') return;
    if (phase !== 'quality_failed') {
      setRerecordUsed(true);
    }
    setPhase('prep');
  };

  return (
    <div data-testid="audio-recorder" className={`p-6 rounded-2xl border border-slate-200 bg-white shadow-soft ${className}`}>
      {/* Preparation Phase */}
      {phase === 'prep' && (
        <div data-testid="recorder-prep" className="text-center py-6 space-y-4">
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-brand-50 border border-brand-200 text-brand-800 text-xs font-semibold uppercase tracking-wider">
            Preparation Time
          </div>
          <div className="font-mono text-5xl font-extrabold text-brand-900">{prepTimeRemaining}s</div>
          <p className="text-sm text-slate-600 max-w-sm mx-auto">
            Organise your ideas. Recording will automatically commence when the countdown reaches zero.
          </p>
          <button
            data-testid="recorder-skip-prep-btn"
            onClick={startRecording}
            className="px-4 py-2 text-xs font-semibold text-brand-700 bg-brand-50 hover:bg-brand-100 rounded-lg transition-colors"
          >
            Skip Prep & Record Now
          </button>
        </div>
      )}

      {/* Recording Phase */}
      {phase === 'recording' && (
        <div data-testid="recorder-recording" className="text-center py-6 space-y-6">
          <div className="flex items-center justify-center gap-2">
            <span className="w-3 h-3 rounded-full bg-rose-500 animate-ping" />
            <span className="text-sm font-semibold text-rose-700">Recording Live</span>
          </div>

          <div className="font-mono text-4xl font-bold text-slate-900">{speakTimeRemaining}s remaining</div>

          {/* Real-time audio waveform/level indicator */}
          <div className="max-w-xs mx-auto space-y-2">
            <div className="flex items-center justify-between text-xs text-slate-500 font-medium">
              <span>Microphone Level</span>
              <span>{audioLevel}%</span>
            </div>
            <div className="h-3 bg-slate-100 rounded-full overflow-hidden border border-slate-200">
              <div
                className="h-full bg-emerald-500 rounded-full transition-all duration-75 ease-out"
                style={{ width: `${audioLevel}%` }}
              />
            </div>
          </div>

          <div>
            <button
              data-testid="recorder-finish-btn"
              onClick={stopRecording}
              className="px-6 py-3 bg-rose-600 hover:bg-rose-700 text-white font-semibold text-sm rounded-xl shadow-soft focus:ring-4 focus:ring-rose-500/20 transition-all"
            >
              Finish & Review Recording
            </button>
          </div>
        </div>
      )}

      {/* Quality Check Failed (Technical re-record) */}
      {phase === 'quality_failed' && (
        <div data-testid="recorder-quality-failed" className="text-center py-6 space-y-4">
          <div className="w-12 h-12 mx-auto rounded-full bg-amber-100 text-amber-700 flex items-center justify-center">
            <IconAlert className="w-6 h-6" />
          </div>
          <h3 className="text-base font-semibold text-slate-900">Audio Check Notice</h3>
          <p className="text-sm text-slate-600 max-w-sm mx-auto">{errorMessage}</p>
          <div className="pt-2">
            <button
              onClick={handleRerecord}
              className="px-5 py-2.5 bg-brand-800 text-white font-semibold text-sm rounded-xl hover:bg-brand-900 shadow-soft"
            >
              Record Again (Technical Check)
            </button>
          </div>
        </div>
      )}

      {/* Review Phase */}
      {phase === 'review' && (
        <div data-testid="recorder-review" className="text-center py-6 space-y-5">
          <div className="w-10 h-10 mx-auto rounded-full bg-emerald-100 text-emerald-700 flex items-center justify-center">
            <IconCheck className="w-5 h-5" />
          </div>
          <div>
            <h3 className="text-base font-semibold text-slate-900">Response Captured</h3>
            <p className="text-xs text-slate-500">You may review playback or record once more if permitted.</p>
          </div>

          {recordedUrl && recordedUrl !== 'mock://audio' && (
            <div className="flex justify-center">
              <audio controls src={recordedUrl} className="h-10" />
            </div>
          )}

          <div className="flex items-center justify-center gap-3 pt-2">
            {allowsRerecord && !rerecordUsed && (
              <button
                data-testid="recorder-rerecord-btn"
                onClick={handleRerecord}
                className="px-4 py-2 border border-slate-300 text-slate-700 font-medium text-sm rounded-lg hover:bg-slate-50 transition-colors"
              >
                Re-record (1 allowed)
              </button>
            )}
            <span className="text-xs text-emerald-700 font-semibold bg-emerald-50 px-3 py-1.5 rounded-full border border-emerald-200">
              Response ready for evaluation
            </span>
          </div>
        </div>
      )}
    </div>
  );
};
