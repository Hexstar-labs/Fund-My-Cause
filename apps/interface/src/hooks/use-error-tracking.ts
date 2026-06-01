import { errorTracker, ErrorContext } from '@/lib/error-tracking';

/**
 * Error tracking hook for React components
 */

export function useErrorTracking() {
  const captureError = (error: Error | string, context?: ErrorContext) => {
    return errorTracker.captureError(error, context);
  };

  const setContext = (context: Partial<ErrorContext>) => {
    errorTracker.setContext(context);
  };

  const getErrors = () => {
    return errorTracker.getErrors();
  };

  const getAnalytics = () => {
    return errorTracker.getErrorAnalytics();
  };

  const reportErrors = (endpoint: string) => {
    return errorTracker.reportErrors(endpoint);
  };

  return {
    captureError,
    setContext,
    getErrors,
    getAnalytics,
    reportErrors,
  };
}
