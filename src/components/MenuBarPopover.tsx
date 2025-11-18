import React, { useEffect, useState } from "react";
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

  // Close popover when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      if (!target.closest(".menu-bar-popover")) {
        onClose?.();
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

  const handlePauseResume = async () => {
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
  };

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
    <div className="menu-bar-popover">
      <div className="cycle-info">
        <div className="cycle-count">
          Ciclo {cycleState.cycle_count}
        </div>
        <div className="phase-name">{getPhaseLabel(cycleState.phase)}</div>
        <div className="timer">{formatTime(cycleState.remaining_seconds)}</div>
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
