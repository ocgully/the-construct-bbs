import { StatusBar, WarningLevel } from './status-bar';

export class SessionTimer {
    private statusBar: StatusBar;
    private remainingSeconds: number = 0;
    private intervalId: number | null = null;
    private isUnlimited: boolean = false;

    constructor(statusBar: StatusBar) {
        this.statusBar = statusBar;
    }

    /**
     * Called when server sends a timer update.
     * Server sends per-minute updates normally, per-second in final minute.
     */
    updateFromServer(data: {
        remaining: number;
        unit: string;
        warning: string;
        handle: string;
        online: number;
        has_mail?: boolean;
    }) {
        if (data.unit === 'unlimited') {
            this.isUnlimited = true;
            this.statusBar.update({
                handle: data.handle,
                online: data.online,
                timeDisplay: 'Unlimited',
                warning: 'normal',
                hasMail: data.has_mail || false,
            });
            this.statusBar.show();
            return;
        }

        // Convert server value to seconds
        if (data.unit === 'min') {
            this.remainingSeconds = data.remaining * 60;
        } else {
            this.remainingSeconds = data.remaining;
        }

        const warning = data.warning as WarningLevel;

        // Update status bar
        this.statusBar.update({
            handle: data.handle,
            online: data.online,
            timeDisplay: this.formatTime(),
            warning: warning,
            hasMail: data.has_mail || false,
        });
        this.statusBar.show();

        // Restart client-side countdown with appropriate tick rate
        this.restartCountdown(data.unit === 'sec');
    }

    /**
     * Start or restart the client-side countdown.
     * Normal mode: tick per minute.
     * Last minute mode: tick per second.
     */
    private restartCountdown(perSecond: boolean) {
        // Clear existing interval to prevent duplicates
        if (this.intervalId !== null) {
            clearInterval(this.intervalId);
            this.intervalId = null;
        }

        if (this.isUnlimited) return;

        const tickMs = perSecond ? 1000 : 60000;
        const decrement = perSecond ? 1 : 60;

        this.intervalId = window.setInterval(() => {
            this.remainingSeconds -= decrement;
            if (this.remainingSeconds < 0) this.remainingSeconds = 0;

            // Determine warning level client-side
            const minutes = this.remainingSeconds / 60;
            let warning: WarningLevel = 'normal';
            if (minutes <= 1) {
                warning = 'red';
            } else if (minutes <= 5) {
                warning = 'yellow';
            }

            // Switch to per-second when hitting 1 minute (if not already)
            if (!perSecond && this.remainingSeconds <= 60 && this.remainingSeconds > 0) {
                this.restartCountdown(true);
                return;
            }

            this.statusBar.update({
                timeDisplay: this.formatTime(),
                warning: warning,
            });
        }, tickMs);
    }

    /**
     * Format remaining time for display.
     */
    private formatTime(): string {
        if (this.isUnlimited) return 'Unlimited';

        const totalSeconds = Math.max(0, this.remainingSeconds);
        const minutes = Math.floor(totalSeconds / 60);
        const seconds = totalSeconds % 60;

        if (minutes > 0) {
            return `${minutes}m`;
        } else {
            return `${seconds}s`;
        }
    }

    /**
     * Stop the timer (on disconnect/logout).
     */
    stop() {
        if (this.intervalId !== null) {
            clearInterval(this.intervalId);
            this.intervalId = null;
        }
        this.statusBar.hide();
    }

    /**
     * Update the online count (can be updated independently of timer).
     */
    updateOnlineCount(count: number) {
        this.statusBar.update({ online: count });
    }

    /**
     * Re-render the status bar (call after terminal writes that may clear the screen).
     */
    refreshBar() {
        this.statusBar.refresh();
    }
}
