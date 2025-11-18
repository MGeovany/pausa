import React, { useEffect, useState, useCallback, useMemo } from "react";
import { Play, Pause } from "lucide-react";
import { useCycleState } from "../store";
import { CycleManager } from "../lib/cycleCommands";
import { toastManager } from "../lib/toastManager";

interface MenuBarPopoverProps {
  onClose?: () => void;
}

const formatTime = (seconds: number): string => {
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes.toString().padStart(2, "0")}:${remainingSeconds
    .toString()
    .padStart(2, "0")}`;
};

const getPhaseLabel = (phase: string): string => {
  switch (phase) {
    case "focus":
      return "Focus Time";
    case "short_break":
      return "Short Break";
    case "long_break":
      return "Long Break";
    default:
      return "Idle";
  }
};

export const MenuBarPopover: React.FC<MenuBarPopoverProps> = ({ onClose }) => {
  const cycleState = useCycleState();
  const [isLoading, setIsLoading] = useState(false);
  const [isVisible, setIsVisible] = useState(false);

  // Fade in animation on mount
  useEffect(() => {
    const timer = setTimeout(() => setIsVisible(true), 10);
    return () => clearTimeout(timer);
  }, []);

  // Memoize phase color calculation
  const phaseColor = useMemo(() => {
    if (!cycleState) return {
      bg: "bg-gray-500/20",
      border: "border-gray-500/30",
      text: "text-gray-300",
      indicator: "bg-gray-400",
    };

    switch (cycleState.phase) {
      case "focus":
        return {
          bg: "bg-blue-500/20",
          border: "border-blue-500/30",
          text: "text-blue-300",
          indicator: "bg-blue-400",
        };
      case "short_break":
        return {
          bg: "bg-green-500/20",
          border: "border-green-500/30",
          text: "text-green-300",
          indicator: "bg-green-400",
        };
      case "long_break":
        return {
          bg: "bg-amber-500/20",
          border: "border-amber-500/30",
          text: "text-amber-300",
          indicator: "bg-amber-400",
        };
      default:
        return {
          bg: "bg-gray-500/20",
          border: "border-gray-500/30",
          text: "text-gray-300",
          indicator: "bg-gray-400",
        };
    }
  }, [cycleState?.phase]);

  // Memoize formatted time
  const formattedTime = useMemo(() => {
    if (!cycleState) return "--:--";
    return formatTime(cycleState.remaining_seconds);
  }, [cycleState?.remaining_seconds]);

  // Memoize phase label
  const phaseLabel = useMemo(() => {
    if (!cycleState) return "Idle";
    return getPhaseLabel(cycleState.phase);
  }, [cycleState?.phase]);

  // Use useCallback for event handlers
  const handlePauseResume = useCallback(async () => {
    if (!cycleState) return;

    setIsLoading(true);
    try {
      if (cycleState.is_running) {
        await CycleManager.pause();
      } else {
        await CycleManager.resume();
      }
    } catch (error) {
      console.error("Failed to pause/resume cycle:", error);
      toastManager.showError(
        "Failed to pause/resume cycle. Please try again.",
        { title: "Cycle Control Failed" }
      );
    } finally {
      setIsLoading(false);
    }
  }, [cycleState?.is_running]);

  // Close popover when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      if (!target.closest(".menu-bar-popover")) {
        setIsVisible(false);
        setTimeout(() => onClose?.(), 200); // Wait for fade out animation
      }
    };

    // Add a small delay to prevent immediate closing
    const timeoutId = setTimeout(() => {
      document.addEventListener("mousedown", handleClickOutside);
    }, 100);

    return () => {
      clearTimeout(timeoutId);
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [onClose]);

  if (!cycleState) {
    return (
      <div className="menu-bar-popover">
        <div className="text-center text-gray-400 text-sm">
          No active cycle
        </div>
      </div>
    );
  }

  return (
    <div 
      className={`menu-bar-popover transition-all duration-300 ease-out ${
        isVisible ? 'opacity-100 scale-100' : 'opacity-0 scale-95'
      }`}
    >
      {/* Phase indicator badge */}
      <div className="flex items-center justify-center mb-3">
        <div className={`inline-flex items-center gap-2 px-3 py-1.5 ${phaseColor.bg} border ${phaseColor.border} rounded-full transition-all duration-300`}>
          <span className={`w-2 h-2 ${phaseColor.indicator} rounded-full animate-pulse`}></span>
          <span className={`text-xs font-medium ${phaseColor.text}`}>
            {phaseLabel}
          </span>
        </div>
      </div>

      <div className="cycle-info">
        <div className="cycle-count">
          Ciclo {cycleState.cycle_count}
        </div>
        <div className="timer">{formattedTime}</div>
      </div>

      <div className="controls">
        <button
          onClick={handlePauseResume}
          disabled={isLoading}
          className="control-button"
        >
          {isLoading ? (
            <span>Loading...</span>
          ) : cycleState.is_running ? (
            <>
              <Pause className="w-4 h-4" />
              <span>Pausar</span>
            </>
          ) : (
            <>
              <Play className="w-4 h-4" />
              <span>Reanudar</span>
            </>
          )}
        </button>
      </div>
    </div>
  );
};
