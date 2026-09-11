export const READINESS_TARGETS: Record<
  string,
  { title: string; safeUse: string; disclaimer: string; currencyNote?: string }
> = {
  OET: {
    title: 'OET Preparation Readiness',
    safeUse:
      'This diagnostic profile highlights general English foundations and observed skill gaps prior to beginning profession-specific OET healthcare communication preparation.',
    disclaimer: 'GEPA does not predict official exam scores.',
    currencyNote:
      'Currency Note: OET remains a four-subtest healthcare-specific assessment; GEPA describes general-English readiness only.',
  },
  IELTS: {
    title: 'IELTS Preparation Readiness',
    safeUse:
      'This diagnostic profile indicates whether foundational English development or exam-specific task practice is likely to be your primary focus.',
    disclaimer: 'GEPA does not predict official exam scores.',
  },
  'TOEFL iBT': {
    title: 'TOEFL iBT Preparation Readiness',
    safeUse:
      'Use this diagnostic profile to assess readiness for academic English tasks and integrated-skill practice formats.',
    disclaimer: 'GEPA does not predict official exam scores.',
    currencyNote:
      'Currency Note: TOEFL iBT changed January 2026: 1–6 section/overall scale and multistage adaptive Reading/Listening.',
  },
  'PTE Academic': {
    title: 'PTE Academic Readiness',
    safeUse:
      'This profile provides insight into linguistic foundation, spoken fluency, and prompt comprehension for computer-delivered task formats.',
    disclaimer: 'GEPA does not predict official exam scores.',
    currencyNote:
      'Currency Note: PTE added Summarize Group Discussion and Respond to a Situation from August 2025.',
  },
  'Cambridge/DET/other': {
    title: 'Cambridge / DET / Other Assessment Preparation',
    safeUse:
      'This profile assists in selecting an appropriate learning level and targeting communicative priorities.',
    disclaimer: 'GEPA does not predict official exam scores.',
  },
  University: {
    title: 'University Study Readiness',
    safeUse:
      'This profile indicates communicative readiness for English-medium lectures, coursework reading, and seminar contributions.',
    disclaimer: 'GEPA does not predict official exam scores.',
  },
  Work: {
    title: 'Workplace Communication Readiness',
    safeUse:
      'This profile highlights operational readiness for workplace emails, meetings, presentations, and professional interactions.',
    disclaimer: 'GEPA does not predict official exam scores.',
  },
  General: {
    title: 'General English Placement & Development',
    safeUse:
      'This diagnostic profile identifies strengths across receptive and productive skills to guide self-directed study and course selection.',
    disclaimer: 'GEPA does not predict official exam scores.',
  },
};
