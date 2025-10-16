import React, { useState, useEffect } from 'react';

interface EmergencyOverrideProps {
  isOpen: boolean;
  onClose: () => void;
  onOverride: (pin: string) => Promise<boolean>;
  emergencyWindowSeconds?: number;
}

export function EmergencyOverride({ 
  isOpen, 
  onClose, 
  onOverride, 
  emergencyWindowSeconds = 45 
}: EmergencyOverrideProps) {
  const [pin, setPin] = useState('');
  const [error, setError] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [remainingTime, setRemainingTime] = useState(emergencyWindowSeconds);
  const [isOverrideActive, setIsOverrideActive] = useState(false);

  // Reset state when modal opens/closes
  useEffect(() => {
    if (isOpen) {
      setPin('');
      setError('');
      setIsProcessing(false);
      setRemainingTime(emergencyWindowSeconds);
      setIsOverrideActive(false);
    }
  }, [isOpen, emergencyWindowSeconds]);

  // Countdown timer for emergency window
  useEffect(() => {
    if (!isOverrideActive || remainingTime <= 0) return;

    const timer = setInterval(() => {
      setRemainingTime((prev) => {
        if (prev <= 1) {
          setIsOverrideActive(false);
          onClose();
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [isOverrideActive, remainingTime, onClose]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!pin.trim() || isProcessing) return;

    setIsProcessing(true);
    setError('');

    try {
      const success = await onOverride(pin);
      if (success) {
        setIsOverrideActive(true);
        setRemainingTime(emergencyWindowSeconds);
        // Don't close immediately - let the countdown run
      } else {
        setError('Invalid PIN. Please try again.');
        setPin('');
      }
    } catch (error) {
      setError('Failed to verify PIN. Please try again.');
      setPin('');
    } finally {
      setIsProcessing(false);
    }
  };

  const handleClose = () => {
    if (!isProcessing) {
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-[60] bg-black/70 flex items-center justify-center p-4">
      <div className="bg-gray-800 rounded-2xl p-8 max-w-md w-full border border-gray-700">
        {!isOverrideActive ? (
          // PIN Entry Phase
          <>
            <div className="flex items-center mb-6">
              <div className="w-12 h-12 bg-red-500/20 rounded-full flex items-center justify-center mr-4">
                <svg className="w-6 h-6 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
                </svg>
              </div>
              <div>
                <h3 className="text-xl font-semibold text-white">Emergency Override</h3>
                <p className="text-gray-400 text-sm">This action will be logged</p>
              </div>
            </div>

            <p className="text-gray-300 mb-6 leading-relaxed">
              Enter your PIN to temporarily override the break. You'll have {emergencyWindowSeconds} seconds 
              to handle your emergency before the break resumes.
            </p>

            <form onSubmit={handleSubmit}>
              <div className="mb-4">
                <input
                  type="password"
                  value={pin}
                  onChange={(e) => setPin(e.target.value)}
                  placeholder="Enter your PIN"
                  className="w-full px-4 py-3 bg-gray-700 text-white rounded-xl border border-gray-600 focus:border-red-500 focus:outline-none text-center text-lg tracking-widest"
                  autoFocus
                  maxLength={6}
                />
              </div>

              {error && (
                <div className="mb-4 p-3 bg-red-500/20 border border-red-500/30 rounded-lg">
                  <p className="text-red-400 text-sm text-center">{error}</p>
                </div>
              )}

              <div className="flex space-x-3">
                <button
                  type="button"
                  onClick={handleClose}
                  disabled={isProcessing}
                  className="flex-1 px-4 py-3 bg-gray-700 hover:bg-gray-600 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-xl transition-colors duration-200"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={isProcessing || !pin.trim()}
                  className="flex-1 px-4 py-3 bg-red-600 hover:bg-red-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-xl transition-colors duration-200 font-medium"
                >
                  {isProcessing ? 'Verifying...' : 'Override'}
                </button>
              </div>
            </form>
          </>
        ) : (
          // Emergency Window Active Phase
          <>
            <div className="text-center">
              <div className="w-16 h-16 bg-green-500/20 rounded-full flex items-center justify-center mx-auto mb-6">
                <svg className="w-8 h-8 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
              </div>

              <h3 className="text-2xl font-semibold text-white mb-2">Emergency Override Active</h3>
              <p className="text-gray-300 mb-8">
                You have temporary access to handle your emergency.
              </p>

              <div className="mb-8">
                <div className="text-6xl font-mono font-light text-white mb-2">
                  {Math.floor(remainingTime / 60).toString().padStart(2, '0')}:
                  {(remainingTime % 60).toString().padStart(2, '0')}
                </div>
                <div className="text-gray-400">
                  Time remaining
                </div>
              </div>

              <div className="bg-yellow-500/20 border border-yellow-500/30 rounded-lg p-4 mb-6">
                <p className="text-yellow-200 text-sm">
                  <strong>Warning:</strong> This override is being logged for security purposes. 
                  The break will automatically resume when the timer expires.
                </p>
              </div>

              <button
                onClick={handleClose}
                className="px-6 py-3 bg-gray-700 hover:bg-gray-600 text-white rounded-xl transition-colors duration-200"
              >
                Resume Break Now
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
