export enum CRTLevel {
  CLEAN = 'crt-clean',
  SUBTLE = 'crt-subtle',
  FULL = 'crt-full',
}

const CRT_STORAGE_KEY = 'bbs-crt-level';

export class CRTController {
  private container: HTMLElement;
  private currentLevel: CRTLevel;

  constructor(container: HTMLElement) {
    this.container = container;

    // Load saved preference or default to FULL
    const saved = localStorage.getItem(CRT_STORAGE_KEY) as CRTLevel;
    this.currentLevel = saved || CRTLevel.FULL;

    // Apply initial level
    this.applyLevel(this.currentLevel);
  }

  getLevel(): CRTLevel {
    return this.currentLevel;
  }

  setLevel(level: CRTLevel): void {
    this.currentLevel = level;
    this.applyLevel(level);
    localStorage.setItem(CRT_STORAGE_KEY, level);
  }

  cycle(): void {
    // Cycle through: FULL -> SUBTLE -> CLEAN -> FULL
    const levels = [CRTLevel.FULL, CRTLevel.SUBTLE, CRTLevel.CLEAN];
    const currentIndex = levels.indexOf(this.currentLevel);
    const nextIndex = (currentIndex + 1) % levels.length;
    this.setLevel(levels[nextIndex]);
  }

  private applyLevel(level: CRTLevel): void {
    // Remove all CRT classes
    this.container.classList.remove(
      CRTLevel.CLEAN,
      CRTLevel.SUBTLE,
      CRTLevel.FULL
    );

    // Add the new level class
    this.container.classList.add(level);

    console.log(`CRT effect level: ${level}`);
  }
}
