import { errorTracker } from '@/lib/error-tracking';
import { useErrorTracking } from '@/hooks/use-error-tracking';

describe('Error Tracking', () => {
  beforeEach(() => {
    errorTracker.clear();
  });

  describe('errorTracker', () => {
    it('should capture errors', () => {
      const error = new Error('Test error');
      errorTracker.captureError(error);

      const errors = errorTracker.getErrors();
      expect(errors).toHaveLength(1);
      expect(errors[0].message).toBe('Test error');
    });

    it('should capture string errors', () => {
      errorTracker.captureError('String error');

      const errors = errorTracker.getErrors();
      expect(errors).toHaveLength(1);
      expect(errors[0].message).toBe('String error');
    });

    it('should set and clear context', () => {
      errorTracker.setContext({ userId: 'user123' });
      errorTracker.captureError('Error with context');

      const errors = errorTracker.getErrors();
      expect(errors[0].context.userId).toBe('user123');

      errorTracker.clearContext();
      errorTracker.captureError('Error without context');

      const allErrors = errorTracker.getErrors();
      expect(allErrors[1].context.userId).toBeUndefined();
    });

    it('should generate analytics', () => {
      errorTracker.captureError('Error: Type A');
      errorTracker.captureError('Error: Type A');
      errorTracker.captureError('Error: Type B');

      const analytics = errorTracker.getErrorAnalytics();
      expect(analytics.total).toBe(3);
      expect(analytics.byType['Error']).toBe(3);
    });

    it('should maintain max error limit', () => {
      for (let i = 0; i < 150; i++) {
        errorTracker.captureError(`Error ${i}`);
      }

      const errors = errorTracker.getErrors();
      expect(errors.length).toBeLessThanOrEqual(100);
    });
  });

  describe('useErrorTracking hook', () => {
    it('should provide error tracking methods', () => {
      const { captureError, getErrors } = useErrorTracking();

      captureError('Hook error');
      const errors = getErrors();

      expect(errors).toHaveLength(1);
      expect(errors[0].message).toBe('Hook error');
    });
  });
});
