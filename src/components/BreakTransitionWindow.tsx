import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface BreakTransitionWindowProps {
  initialCountdown?: number;
}

export const BreakTransitionWindow: React.FC<BreakTransitionWindowProps> = ({
  initialCountdown = 10,
}) => {
  const [countdown, setCountdown] = useState(initialCountdown);
  const [isStopped, setIsStopped] = useState(false);
  const [isVisible, setIsVisible] = useState(false);

  // Fade in animation on mount
  useEffect(() => {
    const timer = setTimeout(() => setIsVisible(true), 10);
    return () => clearTimeout(timer);
  }, []);

  useEffect(() => {
    if (isStopped) return;

    if (countdown === 0) {
      setIsVisible(false);
      setTimeout(() => handleStartBreak(), 300); // Wait for fade out
      return;
    }

    const timer = setTimeout(() => {
      setCountdown(countdown - 1);
    }, 1000);

    return () => clearTimeout(timer);
  }, [countdown, isStopped]);

  const handleStopCountdown = useCallback(async () => {
    setIsStopped(true);
    try {
      await invoke('stop_break_transition_countdown');
    } catch (error) {
      console.error('Failed to stop break transition countdown:', error);
    }
  }, []);

  const handleStartBreak = useCallback(async () => {
    try {
      await invoke('start_break_from_transition');
    } catch (error) {
      console.error('Failed to start break from transition:', error);
    }
  }, []);

  return (
    <div 
      className={`break-transition-window transition-all duration-500 ease-out ${
        isVisible ? 'opacity-100 scale-100' : 'opacity-0 scale-95'
      }`}
    >
      <h2 className="transition-all duration-300">Guarda tus cambios</h2>
      <p className="transition-all duration-300">Tu break empieza en</p>
      <div className={`countdown transition-all duration-300 ${countdown <= 3 ? 'text-red-500 animate-pulse' : ''}`}>
        {countdown}
      </div>

      {!isStopped ? (
        <button onClick={handleStopCountdown} className="transition-button">
          Detener contador
        </button>
      ) : (
        <button onClick={handleStartBreak} className="transition-button animate-pulse-glow">
          Iniciar break
        </button>
      )}

      <style>{`
        .break-transition-window {
          width: 400px;
          height: 300px;
          background: #1a1a1a;
          border-radius: 16px;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          padding: 40px;
        }

        .break-transition-window h2 {
          font-size: 24px;
          color: #fff;
          margin-bottom: 16px;
          font-weight: 600;
        }

        .break-transition-window p {
          font-size: 16px;
          color: #888;
          margin-bottom: 24px;
        }

        .countdown {
          font-size: 72px;
          font-weight: 700;
          color: #007AFF;
          margin-bottom: 32px;
          font-family: 'SF Mono', monospace;
        }

        .transition-button {
          padding: 16px 32px;
          background: #007AFF;
          color: white;
          border: none;
          border-radius: 12px;
          font-size: 16px;
          font-weight: 600;
          cursor: pointer;
          transition: background 0.2s ease;
        }

        .transition-button:hover {
          background: #0056b3;
        }

        .transition-button:active {
          transform: scale(0.98);
        }
      `}</style>
    </div>
  );
};
