import { useState, useEffect } from 'react';
import { EmergencyOverride } from './EmergencyOverride';
import { activityCompletionTracker } from '../lib/breakActivities';
import type { BreakSession, BreakActivity } from '../types';

interface BreakOverlayProps {
  breakSession: BreakSession;
  onCompleteBreak: () => void;
  onEmergencyOverride: (pin: string) => Promise<boolean>;
}

interface BreakActivityChecklistProps {
  activity: BreakActivity;
  onChecklistUpdate: (completedItems: boolean[]) => void;
}

function BreakActivityChecklist({ activity, onChecklistUpdate }: BreakActivityChecklistProps) {
  const [completedItems, setCompletedItems] = useState<boolean[]>(
    new Array(activity.checklist.length).fill(false)
  );

  const handleItemToggle = (index: number) => {
    const newCompletedItems = [...completedItems];
    newCompletedItems[index] = !newCompletedItems[index];
    setCompletedItems(newCompletedItems);
    onChecklistUpdate(newCompletedItems);
  };

  return (
    <div className="bg-gray-800/50 backdrop-blur-sm rounded-2xl p-8 max-w-md w-full">
      <h3 className="text-2xl font-semibold text-white mb-2">{activity.title}</h3>
      <p className="text-gray-300 mb-6 leading-relaxed">{activity.description}</p>
      
      <div className="space-y-3">
        {activity.checklist.map((item, index) => (
          <label
            key={index}
            className="flex items-center space-x-3 cursor-pointer group"
          >
            <div className="relative">
              <input
                type="checkbox"
                checked={completedItems[index]}
                onChange={() => handleItemToggle(index)}
                className="sr-only"
              />
              <div
                className={`w-5 h-5 rounded border-2 transition-all duration-200 ${
                  completedItems[index]
                    ? 'bg-blue-500 border-blue-500'
                    : 'border-gray-400 group-hover:border-blue-400'
                }`}
              >
                {completedItems[index] && (
                  <svg
                    className="w-3 h-3 text-white absolute top-0.5 left-0.5"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                  >
                    <path
                      fillRule="evenodd"
                      d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                      clipRule="evenodd"
                    />
                  </svg>
                )}
              </div>
            </div>
            <span
              className={`text-lg transition-all duration-200 ${
                completedItems[index]
                  ? 'text-gray-400 line-through'
                  : 'text-white group-hover:text-blue-200'
              }`}
            >
              {item}
            </span>
          </label>
        ))}
      </div>
    </div>
  );
}

function CountdownTimer({ remaining }: { remaining: number }) {
  const minutes = Math.floor(remaining / 60);
  const seconds = remaining % 60;

  return (
    <div className="text-center mb-8">
      <div className="text-8xl font-light text-white mb-4 font-mono tracking-wider">
        {minutes.toString().padStart(2, '0')}:{seconds.toString().padStart(2, '0')}
      </div>
      <div className="text-xl text-gray-300">
        {remaining > 60 ? 'Take your time to recharge' : 'Break ending soon'}
      </div>
    </div>
  );
}

export function BreakOverlay({ breakSession, onCompleteBreak, onEmergencyOverride }: BreakOverlayProps) {
  const [checklistCompleted, setChecklistCompleted] = useState<boolean[]>([]);
  const [showEmergencyModal, setShowEmergencyModal] = useState(false);

  // Handle escape key to show emergency override (only if allowed)
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && breakSession.allowEmergency) {
        setShowEmergencyModal(true);
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [breakSession.allowEmergency]);

  const handleEmergencyOverride = async (pin: string): Promise<boolean> => {
    const success = await onEmergencyOverride(pin);
    if (success) {
      setShowEmergencyModal(false);
    }
    return success;
  };

  const handleChecklistUpdate = (completedItems: boolean[]) => {
    setChecklistCompleted(completedItems);
    
    // Record completion progress for analytics
    activityCompletionTracker.recordCompletion(
      breakSession.activity.title,
      completedItems,
      breakSession.id
    );
  };

  const canCompleteBreak = breakSession.remaining <= 0 || 
    (checklistCompleted.length > 0 && checklistCompleted.every(item => item));

  return (
    <div className="fixed inset-0 z-50 bg-gray-900 flex items-center justify-center">
      {/* Main break content */}
      <div className="flex flex-col items-center justify-center min-h-screen p-8">
        {/* Break type indicator */}
        <div className="mb-8">
          <div className="inline-flex items-center px-4 py-2 bg-blue-500/20 rounded-full">
            <div className="w-2 h-2 bg-blue-400 rounded-full mr-3 animate-pulse"></div>
            <span className="text-blue-200 font-medium">
              {breakSession.type === 'short' ? 'Short Break' : 'Long Break'}
            </span>
          </div>
        </div>

        {/* Countdown timer */}
        <CountdownTimer remaining={breakSession.remaining} />

        {/* Break activity checklist */}
        <BreakActivityChecklist
          activity={breakSession.activity}
          onChecklistUpdate={handleChecklistUpdate}
        />

        {/* Complete break button */}
        {canCompleteBreak && (
          <button
            onClick={onCompleteBreak}
            className="mt-8 px-8 py-3 bg-green-600 hover:bg-green-700 text-white font-medium rounded-xl transition-colors duration-200"
          >
            Complete Break
          </button>
        )}

        {/* Emergency override hint */}
        {breakSession.allowEmergency && (
          <div className="absolute bottom-8 left-1/2 transform -translate-x-1/2">
            <p className="text-gray-500 text-sm">
              Press <kbd className="px-2 py-1 bg-gray-800 rounded text-xs">Esc</kbd> for emergency override
            </p>
          </div>
        )}
      </div>

      {/* Emergency override modal */}
      <EmergencyOverride
        isOpen={showEmergencyModal}
        onClose={() => setShowEmergencyModal(false)}
        onOverride={handleEmergencyOverride}
        emergencyWindowSeconds={45}
      />
    </div>
  );
}
