import { useState, useEffect } from 'react';
import { EmergencyOverride } from './EmergencyOverride';
import { activityCompletionTracker, breakActivityManager } from '../lib/breakActivities';
import type { BreakSession, BreakActivity, CycleState } from '../types';
import { useCycleState } from '../store';
import { invoke } from '@tauri-apps/api/core';

interface BreakOverlayProps {
  breakSession: BreakSession;
  onCompleteBreak: () => void;
  onEmergencyOverride: (pin: string) => Promise<boolean>;
  cycleState?: CycleState | null;
  userName?: string;
}

interface BreakActivityChecklistProps {
  activity: BreakActivity;
  onChecklistUpdate: (completedItems: boolean[]) => void;
}

function CelebrationAnimation({ breakType }: { breakType: 'short' | 'long' }) {
  return (
    <div className="absolute inset-0 pointer-events-none overflow-hidden">
      {/* Confetti-like particles */}
      {breakType === 'long' && (
        <>
          {Array.from({ length: 20 }).map((_, i) => (
            <div
              key={i}
              className="absolute animate-float"
              style={{
                left: `${Math.random() * 100}%`,
                top: '-10%',
                animationDelay: `${Math.random() * 2}s`,
                animationDuration: `${3 + Math.random() * 2}s`,
              }}
            >
              <div
                className={`w-2 h-2 rounded-full ${
                  ['bg-amber-400', 'bg-yellow-400', 'bg-orange-400', 'bg-red-400'][
                    Math.floor(Math.random() * 4)
                  ]
                }`}
                style={{
                  opacity: 0.6 + Math.random() * 0.4,
                }}
              />
            </div>
          ))}
        </>
      )}
    </div>
  );
}

function DailyProgressStats({ 
  cyclesCompleted, 
  focusMinutes 
}: { 
  cyclesCompleted: number; 
  focusMinutes: number;
}) {
  return (
    <div className="bg-gray-800/30 backdrop-blur-sm rounded-xl p-4 mb-6">
      <h4 className="text-sm text-gray-400 mb-3 text-center">Today's Progress</h4>
      <div className="flex justify-around">
        <div className="text-center">
          <div className="text-3xl font-bold text-blue-400">{cyclesCompleted}</div>
          <div className="text-xs text-gray-400 mt-1">Cycles</div>
        </div>
        <div className="text-center">
          <div className="text-3xl font-bold text-green-400">{focusMinutes}</div>
          <div className="text-xs text-gray-400 mt-1">Minutes</div>
        </div>
      </div>
    </div>
  );
}

function MilestoneAchievement({ 
  cyclesCompleted 
}: { 
  cyclesCompleted: number;
}) {
  const getMilestone = () => {
    if (cyclesCompleted === 1) return { emoji: '🌱', text: 'First Cycle!', color: 'text-green-400' };
    if (cyclesCompleted === 4) return { emoji: '🔥', text: 'On Fire!', color: 'text-orange-400' };
    if (cyclesCompleted === 8) return { emoji: '⚡', text: 'Unstoppable!', color: 'text-yellow-400' };
    if (cyclesCompleted === 12) return { emoji: '🏆', text: 'Champion!', color: 'text-amber-400' };
    if (cyclesCompleted === 16) return { emoji: '💎', text: 'Diamond Focus!', color: 'text-blue-400' };
    if (cyclesCompleted === 20) return { emoji: '👑', text: 'Legendary!', color: 'text-purple-400' };
    if (cyclesCompleted % 10 === 0) return { emoji: '🎯', text: `${cyclesCompleted} Cycles!`, color: 'text-cyan-400' };
    return null;
  };

  const milestone = getMilestone();
  
  if (!milestone) return null;

  return (
    <div className="mb-4 animate-bounce">
      <div className={`inline-flex items-center px-4 py-2 bg-gray-800/50 rounded-full border-2 ${milestone.color} border-current`}>
        <span className="text-2xl mr-2">{milestone.emoji}</span>
        <span className={`font-bold ${milestone.color}`}>{milestone.text}</span>
      </div>
    </div>
  );
}

function MotivationalMessage({ 
  breakType, 
  userName, 
  cyclesCompleted 
}: { 
  breakType: 'short' | 'long'; 
  userName?: string;
  cyclesCompleted: number;
}) {
  const getMotivationalMessage = () => {
    if (breakType === 'long') {
      const messages = [
        userName ? `Outstanding, ${userName}! You're building incredible focus habits.` : "Outstanding! You're building incredible focus habits.",
        userName ? `${userName}, you're on fire! Keep this momentum going.` : "You're on fire! Keep this momentum going.",
        userName ? `Brilliant work, ${userName}. Your dedication is inspiring.` : "Brilliant work. Your dedication is inspiring.",
        userName ? `${userName}, you've earned this break. Recharge and come back stronger.` : "You've earned this break. Recharge and come back stronger.",
      ];
      return messages[Math.floor(Math.random() * messages.length)];
    }
    
    // Short break messages with variety
    const shortMessages = [
      userName ? `Nice work, ${userName}. Keep the focus flowing.` : "Nice work. Keep the focus flowing.",
      userName ? `You're doing great, ${userName}!` : "You're doing great!",
      userName ? `Solid focus, ${userName}. Take a moment to recharge.` : "Solid focus. Take a moment to recharge.",
    ];
    
    if (cyclesCompleted >= 3) {
      return userName 
        ? `Great momentum, ${userName}! You're making real progress.`
        : "Great momentum! You're making real progress.";
    }
    
    return shortMessages[Math.floor(Math.random() * shortMessages.length)];
  };

  return (
    <p className="text-gray-300 leading-relaxed text-lg">
      {getMotivationalMessage()}
    </p>
  );
}

function BreakCompletionInterface({ 
  breakType,
  userName,
  cyclesCompleted,
  focusMinutes,
  onStartNewBlock,
  onEndSession 
}: {
  breakType: 'short' | 'long';
  userName?: string;
  cyclesCompleted: number;
  focusMinutes: number;
  onStartNewBlock: () => void;
  onEndSession: () => void;
}) {
  return (
    <div className="relative">
      <CelebrationAnimation breakType={breakType} />
      
      <div className="bg-gray-800/50 backdrop-blur-sm rounded-2xl p-8 max-w-md w-full text-center relative z-10">
        <div className="mb-6">
          <div className={`text-6xl mb-4 ${breakType === 'long' ? 'animate-bounce' : ''}`}>
            {breakType === 'long' ? '🎉' : '✨'}
          </div>
          
          {/* Milestone achievement badge */}
          <MilestoneAchievement cyclesCompleted={cyclesCompleted} />
          
          <h3 className="text-3xl font-semibold text-white mb-3">
            {breakType === 'long' ? 'Cycle Complete!' : 'Break Complete'}
          </h3>
          <MotivationalMessage 
            breakType={breakType} 
            userName={userName}
            cyclesCompleted={cyclesCompleted}
          />
        </div>

        {/* Show stats for long breaks */}
        {breakType === 'long' && (
          <DailyProgressStats 
            cyclesCompleted={cyclesCompleted}
            focusMinutes={focusMinutes}
          />
        )}

        <div className="space-y-3">
          <button
            onClick={onStartNewBlock}
            className="w-full px-6 py-4 bg-green-600 hover:bg-green-700 text-white font-medium rounded-xl transition-all duration-200 text-lg transform hover:scale-105"
          >
            Start New Block
          </button>
          <button
            onClick={onEndSession}
            className="w-full px-6 py-4 bg-gray-700 hover:bg-gray-600 text-white font-medium rounded-xl transition-colors duration-200"
          >
            End Day Session
          </button>
        </div>
      </div>
    </div>
  );
}

function CycleProgressIndicator({ 
  cycleCount, 
  cyclesPerLongBreak,
  breakType 
}: { 
  cycleCount: number; 
  cyclesPerLongBreak: number;
  breakType: 'short' | 'long';
}) {
  const completedCycles = breakType === 'long' ? cyclesPerLongBreak : cycleCount % cyclesPerLongBreak;
  
  return (
    <div className="mb-6">
      <div className="flex items-center justify-center space-x-2">
        <span className="text-sm text-gray-400 mr-2">Cycles completed:</span>
        {Array.from({ length: cyclesPerLongBreak }).map((_, index) => (
          <div
            key={index}
            className={`w-3 h-3 rounded-full transition-all duration-300 ${
              index < completedCycles
                ? breakType === 'long' 
                  ? 'bg-amber-400 shadow-lg shadow-amber-400/50' 
                  : 'bg-blue-400 shadow-lg shadow-blue-400/50'
                : 'bg-gray-600'
            }`}
          />
        ))}
        <span className="text-sm text-gray-400 ml-2">
          {completedCycles}/{cyclesPerLongBreak}
        </span>
      </div>
      {breakType === 'long' && (
        <div className="text-center mt-2">
          <span className="text-amber-300 text-sm font-medium">
            🎉 Long break earned!
          </span>
        </div>
      )}
    </div>
  );
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

function CountdownTimer({ 
  remaining, 
  breakType, 
  userName 
}: { 
  remaining: number; 
  breakType: 'short' | 'long';
  userName?: string;
}) {
  const minutes = Math.floor(remaining / 60);
  const seconds = remaining % 60;

  // Different messages based on break type and time remaining
  const getMessage = () => {
    if (breakType === 'long') {
      if (remaining > 600) return userName ? `Great work, ${userName}. Take a proper rest` : 'Take a proper rest';
      if (remaining > 60) return 'Enjoy your long break';
      return 'Long break ending soon';
    } else {
      if (remaining > 60) return 'Take your time to recharge';
      return 'Break ending soon';
    }
  };

  return (
    <div className="text-center mb-8">
      <div className={`font-light text-white mb-4 font-mono tracking-wider ${
        breakType === 'long' ? 'text-9xl' : 'text-8xl'
      }`}>
        {minutes.toString().padStart(2, '0')}:{seconds.toString().padStart(2, '0')}
      </div>
      <div className="text-xl text-gray-300">
        {getMessage()}
      </div>
    </div>
  );
}

export function BreakOverlay({ 
  breakSession, 
  onCompleteBreak, 
  onEmergencyOverride,
  cycleState,
  userName 
}: BreakOverlayProps) {
  const [checklistCompleted, setChecklistCompleted] = useState<boolean[]>([]);
  const [showEmergencyModal, setShowEmergencyModal] = useState(false);
  const [bypassAttempts, setBypassAttempts] = useState(0);
  const storeCycleState = useCycleState();
  
  // Use provided cycleState or fall back to store
  const currentCycleState = cycleState || storeCycleState;

  // Get appropriate activity based on break type
  const activity = breakSession.activity || 
    breakActivityManager.getActivityForBreak(breakSession.type, breakSession.duration);

  // Log bypass attempts
  const logBypassAttempt = async (method: string) => {
    const timestamp = new Date().toISOString();
    console.warn(`[BYPASS ATTEMPT] ${timestamp} - Method: ${method} - Session: ${breakSession.id}`);
    setBypassAttempts(prev => prev + 1);
    
    // Send to backend for persistent logging
    try {
      await invoke('log_bypass_attempt', { 
        sessionId: breakSession.id, 
        method, 
        timestamp 
      });
    } catch (error) {
      console.error('Failed to log bypass attempt:', error);
    }
  };

  // Handle escape key to show emergency override (only if allowed)
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Log any attempt to use keyboard shortcuts during strict mode
      if (event.key === 'Escape') {
        if (breakSession.allowEmergency) {
          setShowEmergencyModal(true);
        } else {
          logBypassAttempt('escape_key');
        }
      }
      
      // Prevent common bypass shortcuts in strict mode
      if (!breakSession.allowEmergency) {
        // Prevent Alt+F4, Cmd+Q, Cmd+W, etc.
        if (
          (event.altKey && event.key === 'F4') ||
          (event.metaKey && (event.key === 'q' || event.key === 'w')) ||
          (event.ctrlKey && event.key === 'w')
        ) {
          event.preventDefault();
          logBypassAttempt(`keyboard_shortcut_${event.key}`);
        }
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [breakSession.allowEmergency, breakSession.id]);

  const handleEmergencyOverride = async (pin: string): Promise<boolean> => {
    const success = await onEmergencyOverride(pin);
    if (success) {
      setShowEmergencyModal(false);
      logBypassAttempt('emergency_override_success');
    } else {
      logBypassAttempt('emergency_override_failed');
    }
    return success;
  };

  const handleChecklistUpdate = (completedItems: boolean[]) => {
    setChecklistCompleted(completedItems);
    
    // Record completion progress for analytics
    activityCompletionTracker.recordCompletion(
      activity.title,
      completedItems,
      breakSession.id
    );
  };

  const canCompleteBreak = breakSession.remaining <= 0 || 
    (checklistCompleted.length > 0 && checklistCompleted.every(item => item));
  
  const showCompletionInterface = breakSession.remaining <= 0;

  // Different background colors for break types
  const bgColor = breakSession.type === 'long' 
    ? 'bg-gradient-to-br from-amber-900/40 via-gray-900 to-gray-900' 
    : 'bg-gray-900';
  
  const accentColor = breakSession.type === 'long' ? 'amber' : 'blue';

  return (
    <div 
      className={`fixed inset-0 z-50 ${bgColor} flex items-center justify-center transition-colors duration-500`}
      style={{ 
        // Ensure fullscreen on all monitors
        width: '100vw',
        height: '100vh',
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0
      }}
    >
      {/* Main break content */}
      <div className="flex flex-col items-center justify-center min-h-screen p-8 w-full">
        {showCompletionInterface ? (
          // Show completion interface when break is over
          <BreakCompletionInterface
            breakType={breakSession.type}
            userName={userName}
            cyclesCompleted={currentCycleState?.cycle_count || 0}
            focusMinutes={Math.floor((currentCycleState?.cycle_count || 0) * 25)} // Approximate
            onStartNewBlock={onCompleteBreak}
            onEndSession={onCompleteBreak}
          />
        ) : (
          <>
            {/* Break type indicator */}
            <div className="mb-8">
              <div className={`inline-flex items-center px-4 py-2 bg-${accentColor}-500/20 rounded-full`}>
                <div className={`w-2 h-2 bg-${accentColor}-400 rounded-full mr-3 animate-pulse`}></div>
                <span className={`text-${accentColor}-200 font-medium`}>
                  {breakSession.type === 'short' ? '☕ Short Break' : '🌟 Long Break'}
                </span>
              </div>
            </div>

            {/* Cycle progress indicator */}
            {currentCycleState && currentCycleState.cycle_count > 0 && (
              <CycleProgressIndicator
                cycleCount={currentCycleState.cycle_count}
                cyclesPerLongBreak={4} // TODO: Get from settings
                breakType={breakSession.type}
              />
            )}

            {/* Countdown timer */}
            <CountdownTimer 
              remaining={breakSession.remaining} 
              breakType={breakSession.type}
              userName={userName}
            />

            {/* Break activity checklist */}
            <BreakActivityChecklist
              activity={activity}
              onChecklistUpdate={handleChecklistUpdate}
            />

            {/* Complete break button (only if checklist is done) */}
            {canCompleteBreak && !showCompletionInterface && (
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

            {/* Bypass attempts indicator (for debugging/awareness) */}
            {bypassAttempts > 0 && (
              <div className="absolute top-8 right-8">
                <div className="bg-red-500/20 border border-red-500/30 rounded-lg px-4 py-2">
                  <p className="text-red-400 text-sm">
                    Bypass attempts: {bypassAttempts}
                  </p>
                </div>
              </div>
            )}
          </>
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
