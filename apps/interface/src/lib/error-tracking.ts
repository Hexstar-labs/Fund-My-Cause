/**
 * Error tracking and reporting service
 * Captures, logs, and reports errors with context
 */

export interface ErrorContext {
  userId?: string;
  campaignId?: string;
  action?: string;
  metadata?: Record<string, unknown>;
}

export interface ErrorReport {
  message: string;
  stack?: string;
  context: ErrorContext;
  timestamp: string;
  url: string;
  userAgent: string;
}

class ErrorTracker {
  private errors: ErrorReport[] = [];
  private maxErrors = 100;
  private context: ErrorContext = {};

  setContext(context: Partial<ErrorContext>) {
    this.context = { ...this.context, ...context };
  }

  clearContext() {
    this.context = {};
  }

  captureError(error: Error | string, context?: ErrorContext) {
    const errorReport: ErrorReport = {
      message: typeof error === 'string' ? error : error.message,
      stack: error instanceof Error ? error.stack : undefined,
      context: { ...this.context, ...context },
      timestamp: new Date().toISOString(),
      url: typeof window !== 'undefined' ? window.location.href : '',
      userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : '',
    };

    this.errors.push(errorReport);
    if (this.errors.length > this.maxErrors) {
      this.errors.shift();
    }

    this.logError(errorReport);
    return errorReport;
  }

  private logError(report: ErrorReport) {
    if (typeof window !== 'undefined') {
      console.error('[ErrorTracker]', {
        message: report.message,
        context: report.context,
        timestamp: report.timestamp,
      });
    }
  }

  getErrors(): ErrorReport[] {
    return [...this.errors];
  }

  getErrorAnalytics() {
    const analytics = {
      total: this.errors.length,
      byType: {} as Record<string, number>,
      recent: this.errors.slice(-10),
    };

    this.errors.forEach((error) => {
      const type = error.message.split(':')[0] || 'Unknown';
      analytics.byType[type] = (analytics.byType[type] || 0) + 1;
    });

    return analytics;
  }

  reportErrors(endpoint: string) {
    if (this.errors.length === 0) return Promise.resolve();

    return fetch(endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        errors: this.errors,
        analytics: this.getErrorAnalytics(),
      }),
    }).catch((err) => {
      console.error('Failed to report errors:', err);
    });
  }

  clear() {
    this.errors = [];
  }
}

export const errorTracker = new ErrorTracker();

// Global error handler
if (typeof window !== 'undefined') {
  window.addEventListener('error', (event) => {
    errorTracker.captureError(event.error || event.message);
  });

  window.addEventListener('unhandledrejection', (event) => {
    errorTracker.captureError(
      event.reason instanceof Error ? event.reason : new Error(String(event.reason))
    );
  });
}
