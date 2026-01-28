import { Terminal } from '@xterm/xterm';

export type WarningLevel = 'normal' | 'yellow' | 'red';

export class StatusBar {
    private terminal: Terminal;
    private handle: string = '';
    private onlineCount: number = 0;
    private timeDisplay: string = '';
    private warningLevel: WarningLevel = 'normal';
    private visible: boolean = false;
    private hasMail: boolean = false;

    constructor(terminal: Terminal) {
        this.terminal = terminal;
    }

    /**
     * Update status bar content and re-render.
     */
    update(opts: {
        handle?: string;
        online?: number;
        timeDisplay?: string;
        warning?: WarningLevel;
        hasMail?: boolean;
    }) {
        if (opts.handle !== undefined) this.handle = opts.handle;
        if (opts.online !== undefined) this.onlineCount = opts.online;
        if (opts.timeDisplay !== undefined) this.timeDisplay = opts.timeDisplay;
        if (opts.warning !== undefined) this.warningLevel = opts.warning;
        if (opts.hasMail !== undefined) this.hasMail = opts.hasMail;
        this.render();
    }

    /**
     * Show the status bar (called after authentication).
     */
    show() {
        this.visible = true;
        this.render();
    }

    /**
     * Hide the status bar (called on disconnect/logout).
     */
    hide() {
        this.visible = false;
        // Clear row 24
        this.terminal.write('\x1b7\x1b[24;1H\x1b[2K\x1b8');
    }

    /**
     * Force a re-render of the status bar (e.g., after screen clears).
     */
    refresh() {
        this.render();
    }

    /**
     * Render the status bar at row 24 using ANSI escape codes.
     * Save cursor -> move to row 24 -> render bar -> restore cursor.
     */
    private render() {
        if (!this.visible) return;

        // Background and foreground colors based on warning level
        let bgColor: string;
        let fgColor: string;
        switch (this.warningLevel) {
            case 'red':
                bgColor = '\x1b[41m';   // Red background
                fgColor = '\x1b[97m';   // Bright white text
                break;
            case 'yellow':
                bgColor = '\x1b[43m';   // Yellow background
                fgColor = '\x1b[30m';   // Black text
                break;
            default:
                bgColor = '\x1b[40m';   // Black background (no color until meaningful)
                fgColor = '\x1b[90m';   // Light grey text
                break;
        }

        // Build the bar content
        const left = ` ${this.handle}`;
        const mailIndicator = this.hasMail ? ' \x1b[33m\x1b[1mMAIL\x1b[0m' : '';
        const center = `Online: ${this.onlineCount}`;
        const right = `Time: ${this.timeDisplay} `;

        // Calculate visible length (excluding ANSI escape codes)
        const mailVisibleLen = this.hasMail ? 5 : 0; // " MAIL" = 5 chars
        const visibleContentLen = left.length + mailVisibleLen + center.length + right.length;
        const totalWidth = 80;

        // Simple approach: left-aligned handle + MAIL, centered online count, right-aligned time
        const spaceBetweenLeftCenter = Math.max(1, Math.floor((totalWidth - visibleContentLen) / 2));
        const spaceBetweenCenterRight = Math.max(1, totalWidth - left.length - mailVisibleLen - spaceBetweenLeftCenter - center.length - right.length);

        // Apply colors before building string
        const barContent = bgColor + fgColor + left + mailIndicator +
            bgColor + fgColor + ' '.repeat(spaceBetweenLeftCenter) +
            center +
            ' '.repeat(spaceBetweenCenterRight) +
            right;

        // Pad to exactly 80 visible chars (account for ANSI codes in mailIndicator)
        const currentVisibleLen = visibleContentLen + spaceBetweenLeftCenter + spaceBetweenCenterRight;
        const paddingNeeded = totalWidth - currentVisibleLen;
        const paddedBar = barContent + bgColor + fgColor + ' '.repeat(Math.max(0, paddingNeeded));

        // Write: save cursor, move to row 24 col 1, write bar, reset, restore cursor
        this.terminal.write(
            '\x1b7' +           // Save cursor position (DEC)
            '\x1b[24;1H' +     // Move to row 24, column 1
            paddedBar +         // Bar content (already has colors embedded)
            '\x1b[0m' +        // Reset colors
            '\x1b8'            // Restore cursor position (DEC)
        );
    }
}
