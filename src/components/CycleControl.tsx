import React from "react";
import { Play, Coffee, Square, Target } from "lucide-react";
import { useCycleState } from "../store";
import { useCycleManager } from "../lib/useCycleManager";

/**
 * Component to control work cycles (start focus, start break, end session)
 */
export const CycleControl: React.FC = () => {
  const cycleState = useCycleState();
  const { startFocusSession, startBreakSession, endSession } =
    useCycleManager();

  const handleStartFocus = async () => {
    try {
      await startFocusSession();
    } catch (error) {
      console.error("Failed to start focus session:", error);
      // TODO: Show error toast
    }
  };

  const handleStartBreak = async () => {
    try {
      await startBreakSession(false);
    } catch (error) {
      console.error("Failed to start break:", error);
      // TODO: Show error toast
    }
  };

  const handleEndSession = async () => {
    try {
      await endSession(false);
    } catch (error) {
      console.error("Failed to end session:", error);
      // TODO: Show error toast
    }
  };

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
        return "Focus Session";
      case "short_break":
        return "Short Break";
      case "long_break":
        return "Long Break";
      default:
        return "Idle";
    }
  };

  if (!cycleState) {
    return (
      <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
        <p className="text-gray-400">Loading cycle state...</p>
      </div>
    );
  }

  return (
    <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold text-white">Work Cycles</h2>
        {cycleState.cycle_count > 0 && (
          <div className="flex items-center space-x-2 bg-gray-700 rounded-full px-3 py-1">
            <Target className="w-4 h-4 text-blue-400" />
            <span className="text-sm font-semibold text-blue-400">
              {cycleState.cycle_count} cycles
            </span>
          </div>
        )}
      </div>

      {/* Current phase display */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-2">
          <span className="text-gray-400 text-sm">Current Phase</span>
          <span
            className={`text-sm font-medium ${
              cycleState.phase === "focus"
                ? "text-blue-400"
                : cycleState.phase === "short_break"
                ? "text-green-400"
                : cycleState.phase === "long_break"
                ? "text-purple-400"
                : "text-gray-400"
            }`}
          >
            {getPhaseLabel(cycleState.phase)}
          </span>
        </div>

        {cycleState.phase !== "idle" && (
          <div className="text-center py-4">
            <div className="text-4xl font-mono font-bold text-white mb-2">
              {formatTime(cycleState.remaining_seconds)}
            </div>
            <div className="text-sm text-gray-400">
              {cycleState.is_running ? "Running" : "Paused"}
            </div>
          </div>
        )}
      </div>

      {/* Control buttons */}
      <div className="flex space-x-3">
        {cycleState.phase === "idle" && (
          <>
            <button
              onClick={handleStartFocus}
              disabled={!cycleState.can_start}
              className="flex-1 flex items-center justify-center space-x-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed text-white font-medium py-3 px-4 rounded-lg transition-colors"
            >
              <Play className="w-5 h-5" />
              <span>Start Focus</span>
            </button>
            <button
              onClick={handleStartBreak}
              className="flex-1 flex items-center justify-center space-x-2 bg-green-600 hover:bg-green-700 text-white font-medium py-3 px-4 rounded-lg transition-colors"
            >
              <Coffee className="w-5 h-5" />
              <span>Start Break</span>
            </button>
          </>
        )}

        {cycleState.phase !== "idle" && (
          <button
            onClick={handleEndSession}
            className="flex-1 flex items-center justify-center space-x-2 bg-red-600 hover:bg-red-700 text-white font-medium py-3 px-4 rounded-lg transition-colors"
          >
            <Square className="w-5 h-5" />
            <span>End Session</span>
          </button>
        )}
      </div>

      {/* Work hours warning */}
      {!cycleState.can_start && cycleState.phase === "idle" && (
        <div className="mt-4 p-3 bg-yellow-900/20 border border-yellow-700/50 rounded-lg">
          <p className="text-sm text-yellow-400">
            ⚠️ Outside work hours. Focus sessions are restricted to your
            configured work schedule.
          </p>
        </div>
      )}
    </div>
  );
};
