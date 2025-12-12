/**
 * Comprehensive error handling and logging utility
 */

export interface ErrorLog {
  timestamp: string;
  level: "error" | "warning" | "info";
  message: string;
  context?: string;
  stack?: string;
  userAction?: string;
  recoverable: boolean;
}

export interface ErrorRecoveryStrategy {
  canRecover: boolean;
  retryable: boolean;
  userMessage: string;
  technicalMessage: string;
  suggestedActions: string[];
}

class ErrorHandler {
  private errorLogs: ErrorLog[] = [];
  private maxLogs = 100;
  private errorCallbacks: ((error: ErrorLog) => void)[] = [];

  /**
   * Log an error with context
   */
  logError(
    error: Error | string,
    context?: string,
    userAction?: string,
    level: "error" | "warning" | "info" = "error"
  ): void {
    const errorLog: ErrorLog = {
      timestamp: new Date().toISOString(),
      level,
      message: typeof error === "string" ? error : error.message,
      context,
      stack: typeof error === "object" ? error.stack : undefined,
      userAction,
      recoverable: this.isRecoverable(error),
    };

    // Add to logs
    this.errorLogs.push(errorLog);

    // Trim logs if exceeding max
    if (this.errorLogs.length > this.maxLogs) {
      this.errorLogs = this.errorLogs.slice(-this.maxLogs);
    }

    // Console logging with formatting
    const logMethod =
      level === "error"
        ? console.error
        : level === "warning"
        ? console.warn
        : console.info;
    logMethod(
      `[${level.toUpperCase()}] ${errorLog.timestamp}`,
      `\nContext: ${context || "N/A"}`,
      `\nMessage: ${errorLog.message}`,
      userAction ? `\nUser Action: ${userAction}` : "",
      errorLog.stack ? `\nStack: ${errorLog.stack}` : ""
    );

    // Notify callbacks
    this.errorCallbacks.forEach((callback) => callback(errorLog));

    // Store in localStorage for persistence
    this.persistErrorLogs();

    // Send to telemetry service (non-blocking)
    this.sendToTelemetry(errorLog);
  }

  /**
   * Determine if an error is recoverable
   */
  private isRecoverable(error: Error | string): boolean {
    const errorMessage = typeof error === "string" ? error : error.message;

    // Non-recoverable errors
    const nonRecoverablePatterns = [
      /database.*corrupt/i,
      /fatal/i,
      /critical/i,
      /permission denied/i,
    ];

    return !nonRecoverablePatterns.some((pattern) =>
      pattern.test(errorMessage)
    );
  }

  /**
   * Get recovery strategy for an error
   */
  getRecoveryStrategy(error: Error | string): ErrorRecoveryStrategy {
    const errorMessage = typeof error === "string" ? error : error.message;
    const errorLower = errorMessage.toLowerCase();

    // Network errors
    if (
      errorLower.includes("network") ||
      errorLower.includes("fetch") ||
      errorLower.includes("connection")
    ) {
      return {
        canRecover: true,
        retryable: true,
        userMessage:
          "Network connection issue. Please check your internet connection.",
        technicalMessage: errorMessage,
        suggestedActions: [
          "Check your internet connection",
          "Try again in a moment",
          "Restart the application if the problem persists",
        ],
      };
    }

    // Validation errors
    if (errorLower.includes("validation") || errorLower.includes("invalid")) {
      return {
        canRecover: true,
        retryable: false,
        userMessage: "Invalid input. Please check your entries and try again.",
        technicalMessage: errorMessage,
        suggestedActions: [
          "Review your input for errors",
          "Ensure all required fields are filled",
          "Check that values are in the correct format",
        ],
      };
    }

    // Work hours errors
    if (
      errorLower.includes("work hours") ||
      errorLower.includes("outside work")
    ) {
      return {
        canRecover: true,
        retryable: false,
        userMessage: "This action is outside your configured work hours.",
        technicalMessage: errorMessage,
        suggestedActions: [
          "Wait until your work hours begin",
          "Override work hours if needed",
          "Adjust your work schedule in settings",
        ],
      };
    }

    // Database errors
    if (errorLower.includes("database") || errorLower.includes("sqlite")) {
      return {
        canRecover: false,
        retryable: false,
        userMessage: "Database error. Your data may need recovery.",
        technicalMessage: errorMessage,
        suggestedActions: [
          "Restart the application",
          "Check available disk space",
          "Contact support if the problem persists",
        ],
      };
    }

    // Onboarding errors
    if (errorLower.includes("onboarding")) {
      return {
        canRecover: true,
        retryable: true,
        userMessage: "Setup error. Please try again.",
        technicalMessage: errorMessage,
        suggestedActions: [
          "Go back and review your settings",
          "Restart the onboarding process",
          "Contact support if you continue to have issues",
        ],
      };
    }

    // Cycle/session errors
    if (errorLower.includes("cycle") || errorLower.includes("session")) {
      return {
        canRecover: true,
        retryable: true,
        userMessage: "Session error. Please try restarting.",
        technicalMessage: errorMessage,
        suggestedActions: [
          "End the current session and start a new one",
          "Check your cycle configuration",
          "Restart the application if needed",
        ],
      };
    }

    // Generic error
    return {
      canRecover: true,
      retryable: true,
      userMessage: "An unexpected error occurred. Please try again.",
      technicalMessage: errorMessage,
      suggestedActions: [
        "Try the action again",
        "Restart the application",
        "Contact support if the problem persists",
      ],
    };
  }

  /**
   * Register a callback for error notifications
   */
  onError(callback: (error: ErrorLog) => void): () => void {
    this.errorCallbacks.push(callback);

    // Return unsubscribe function
    return () => {
      this.errorCallbacks = this.errorCallbacks.filter((cb) => cb !== callback);
    };
  }

  /**
   * Get all error logs
   */
  getErrorLogs(): ErrorLog[] {
    return [...this.errorLogs];
  }

  /**
   * Get recent error logs
   */
  getRecentErrors(count: number = 10): ErrorLog[] {
    return this.errorLogs.slice(-count);
  }

  /**
   * Clear error logs
   */
  clearLogs(): void {
    this.errorLogs = [];
    localStorage.removeItem("pausa-error-logs");
  }

  /**
   * Persist error logs to localStorage
   */
  private persistErrorLogs(): void {
    try {
      const recentLogs = this.errorLogs.slice(-50); // Keep last 50 logs
      localStorage.setItem("pausa-error-logs", JSON.stringify(recentLogs));
    } catch (error) {
      console.warn("Failed to persist error logs:", error);
    }
  }

  /**
   * Load error logs from localStorage
   */
  loadPersistedLogs(): void {
    try {
      const stored = localStorage.getItem("pausa-error-logs");
      if (stored) {
        const logs = JSON.parse(stored) as ErrorLog[];
        this.errorLogs = logs;
      }
    } catch (error) {
      console.warn("Failed to load persisted error logs:", error);
    }
  }

  /**
   * Send error to telemetry service (non-blocking)
   */
  private async sendToTelemetry(errorLog: ErrorLog) {
    try {
      const { tauriCommands } = await import("./tauri");
      await tauriCommands.sendErrorEvent({
        errorType: errorLog.level,
        message: errorLog.message,
        context: errorLog.context,
        stack: errorLog.stack,
        userAction: errorLog.userAction,
        recoverable: errorLog.recoverable,
      });
    } catch (error) {
      // Silently fail - telemetry should not break the app
      console.debug("Failed to send error to telemetry:", error);
    }
  }

  /**
   * Export error logs for debugging
   */
  exportLogs(): string {
    return JSON.stringify(this.errorLogs, null, 2);
  }

  /**
   * Handle automatic recovery attempts
   */
  async attemptRecovery(
    error: Error | string,
    retryFn: () => Promise<void>,
    maxRetries: number = 3
  ): Promise<boolean> {
    const strategy = this.getRecoveryStrategy(error);

    if (!strategy.retryable) {
      this.logError(error, "Recovery", "Not retryable", "warning");
      return false;
    }

    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        this.logError(
          `Recovery attempt ${attempt}/${maxRetries}`,
          "Recovery",
          "Automatic retry",
          "info"
        );

        // Exponential backoff
        await new Promise((resolve) =>
          setTimeout(resolve, Math.pow(2, attempt) * 1000)
        );

        await retryFn();

        this.logError(
          `Recovery successful on attempt ${attempt}`,
          "Recovery",
          "Automatic retry succeeded",
          "info"
        );

        return true;
      } catch (retryError) {
        this.logError(
          retryError as Error,
          "Recovery",
          `Retry attempt ${attempt} failed`,
          "warning"
        );

        if (attempt === maxRetries) {
          return false;
        }
      }
    }

    return false;
  }
}

// Singleton instance
export const errorHandler = new ErrorHandler();

// Load persisted logs on initialization
errorHandler.loadPersistedLogs();

// Global error handler for unhandled errors
if (typeof window !== "undefined") {
  window.addEventListener("error", (event) => {
    errorHandler.logError(
      event.error || event.message,
      "Global",
      "Unhandled error",
      "error"
    );
  });

  window.addEventListener("unhandledrejection", (event) => {
    errorHandler.logError(
      event.reason,
      "Global",
      "Unhandled promise rejection",
      "error"
    );
  });
}

/**
 * Utility function to wrap async functions with error handling
 */
export function withErrorHandling<T extends (...args: any[]) => Promise<any>>(
  fn: T,
  context: string
): T {
  return (async (...args: Parameters<T>) => {
    try {
      return await fn(...args);
    } catch (error) {
      errorHandler.logError(error as Error, context, fn.name);
      throw error;
    }
  }) as T;
}

/**
 * Utility function to show user-friendly error messages
 */
export function getUserFriendlyError(error: Error | string): string {
  const strategy = errorHandler.getRecoveryStrategy(error);
  return strategy.userMessage;
}
