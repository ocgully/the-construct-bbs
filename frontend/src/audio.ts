// Modem handshake sound - Web Audio API playback with user gesture handling
//
// Browser autoplay policy requires a user gesture before AudioContext can play.
// Since we auto-connect on page load (no keypress required), the modem sound
// is triggered on the FIRST user interaction (keypress) after connection.
// This satisfies the browser autoplay policy while maintaining atmosphere.

let audioContext: AudioContext | null = null;
let modemBuffer: AudioBuffer | null = null;
let loaded = false;

export async function loadModemSound(): Promise<void> {
  try {
    const response = await fetch('/audio/modem.mp3');
    if (!response.ok) {
      console.warn('Modem sound not found, skipping audio');
      return;
    }
    const arrayBuffer = await response.arrayBuffer();
    audioContext = new AudioContext();
    modemBuffer = await audioContext.decodeAudioData(arrayBuffer);
    loaded = true;
  } catch (e) {
    console.warn('Failed to load modem sound:', e);
  }
}

export async function playModemSound(): Promise<void> {
  if (!loaded || !audioContext || !modemBuffer) return;

  // Resume AudioContext if suspended (browser autoplay policy)
  if (audioContext.state === 'suspended') {
    await audioContext.resume();
  }

  const source = audioContext.createBufferSource();
  source.buffer = modemBuffer;
  source.connect(audioContext.destination);
  source.start(0);
}

export function isAudioLoaded(): boolean {
  return loaded;
}
