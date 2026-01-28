// Modem handshake sounds - Web Audio API playback
//
// Loads both modem-success.mp3 and modem-fail.mp3 at startup.
// The backend sends a JSON signal { type: "modem", status: "success"|"fail" }
// after determining whether a node was assigned, and the frontend plays the
// appropriate sound via the websocket message handler.
//
// playModemSuccess/Fail return a Promise that resolves when playback ENDS,
// so callers can wait for the sound to finish before proceeding.

let audioContext: AudioContext | null = null;
let successBuffer: AudioBuffer | null = null;
let failBuffer: AudioBuffer | null = null;
let bellBuffer: AudioBuffer | null = null;
let loaded = false;

// Generate a simple terminal bell tone (~800Hz, 150ms) programmatically
// No external file needed - classic BBS bell sound
async function generateBellBuffer(): Promise<AudioBuffer | null> {
  if (!audioContext) return null;

  const sampleRate = audioContext.sampleRate;
  const duration = 0.15; // 150ms
  const frequency = 800; // Hz
  const numSamples = Math.floor(sampleRate * duration);

  const buffer = audioContext.createBuffer(1, numSamples, sampleRate);
  const data = buffer.getChannelData(0);

  for (let i = 0; i < numSamples; i++) {
    // Sine wave with exponential decay envelope
    const t = i / sampleRate;
    const envelope = Math.exp(-t * 20); // Decay factor
    data[i] = Math.sin(2 * Math.PI * frequency * t) * envelope * 0.3;
  }

  return buffer;
}

export async function loadModemSounds(): Promise<void> {
  try {
    audioContext = new AudioContext();

    const [successResp, failResp] = await Promise.all([
      fetch('/audio/modem-success.mp3'),
      fetch('/audio/modem-fail.mp3'),
    ]);

    if (successResp.ok) {
      const buf = await successResp.arrayBuffer();
      successBuffer = await audioContext.decodeAudioData(buf);
    } else {
      console.warn('modem-success.mp3 not found, skipping');
    }

    if (failResp.ok) {
      const buf = await failResp.arrayBuffer();
      failBuffer = await audioContext.decodeAudioData(buf);
    } else {
      console.warn('modem-fail.mp3 not found, skipping');
    }

    // Generate bell sound programmatically (no external file needed)
    bellBuffer = await generateBellBuffer();

    loaded = true;
  } catch (e) {
    console.warn('Failed to load modem sounds:', e);
  }
}

async function playBuffer(buffer: AudioBuffer | null): Promise<void> {
  if (!loaded || !audioContext || !buffer) return;

  // Resume AudioContext if suspended (browser autoplay policy)
  if (audioContext.state === 'suspended') {
    await audioContext.resume();
  }

  // Return a Promise that resolves when the sound finishes playing
  return new Promise<void>((resolve) => {
    const source = audioContext!.createBufferSource();
    source.buffer = buffer;
    source.connect(audioContext!.destination);
    source.onended = () => resolve();
    source.start(0);
  });
}

export async function playModemSuccess(): Promise<void> {
  return playBuffer(successBuffer);
}

export async function playModemFail(): Promise<void> {
  return playBuffer(failBuffer);
}

export async function playBellSound(): Promise<void> {
  return playBuffer(bellBuffer);
}

/// Resume the AudioContext. Must be called during a user gesture (e.g. keydown)
/// to satisfy browser autoplay policy.
export async function ensureAudioReady(): Promise<void> {
  if (!audioContext) return;
  if (audioContext.state === 'suspended') {
    await audioContext.resume();
  }
}
