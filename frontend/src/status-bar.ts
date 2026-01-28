import { Terminal } from '@xterm/xterm';

export type WarningLevel = 'normal' | 'yellow' | 'red';

export class StatusBar {
    private terminal: Terminal;
    private handle: string = '';
    private onlineCount: number = 0;
    private timeDisplay: string = '';
    private warningLevel: WarningLevel = 'normal';
    private visible: boolean = false;

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
    }) {
        if (opts.handle !== undefined) this.handle = opts.handle;
        if (opts.online !== undefined) this.onlineCount = opts.online;
        if (opts.timeDisplay !== undefined) this.timeDisplay = opts.timeDisplay;
        if (opts.warning !== undefined) this.warningLevel = opts.warning;
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
                bgColor = '\x1b[44m';   // Blue background (more visible than black)
                fgColor = '\x1b[97m';   // Bright white text
                break;
        }

        // Build the bar content
        const left = ` ${this.handle}`;
        const center = `Online: ${this.onlineCount}`;
        const right = `Time: ${this.timeDisplay} `;

        // Pad to fill 80 columns
        const contentLen = left.length + center.length + right.length;
        const totalWidth = 80;
        const leftPad = Math.floor((totalWidth - contentLen) / 3);
        const rightPad = totalWidth - left.length - leftPad - center.length - (totalWidth - left.length - leftPad - center.length - right.length);

        // Simple approach: left-aligned handle, centered online count, right-aligned time
        const spaceBetweenLeftCenter = Math.max(1, Math.floor((totalWidth - left.length - center.length - right.length) / 2));
        const spaceBetweenCenterRight = Math.max(1, totalWidth - left.length - spaceBetweenLeftCenter - center.length - right.length);

        const barContent = left +
            ' '.repeat(spaceBetweenLeftCenter) +
            center +
            ' '.repeat(spaceBetweenCenterRight) +
            right;

        // Pad or trim to exactly 80 chars
        const paddedBar = barContent.length < totalWidth
            ? barContent + ' '.repeat(totalWidth - barContent.length)
            : barContent.substring(0, totalWidth);

        // Write: save cursor, move to row 24 col 1, set colors, write bar, reset, restore cursor
        this.terminal.write(
            '\x1b7' +           // Save cursor position (DEC)
            '\x1b[24;1H' +     // Move to row 24, column 1
            bgColor + fgColor + // Set colors
            paddedBar +         // Bar content
            '\x1b[0m' +        // Reset colors
            '\x1b8'            // Restore cursor position (DEC)
        );
    }
}
