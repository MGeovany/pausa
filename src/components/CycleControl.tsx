import React, { useState, useEffect } from "react";
import {
  Play,
  Coffee,
  Square,
  Target,
  Clock,
  AlertCircle,
  TrendingUp,
} from "lucide-react";
import { useCycleState } from "../store";
import { useCycleManager } from "../lib/useCycleManager";
import {
  CycleManager,
  WorkScheduleInfo,
  WorkHoursStats,
  getWorkHoursStats,
} from "../lib/cycleCommands";

/**
 * Component to control work cycles (start focus, start break, end session)
 */
export const CycleControl: React.FC = () => {
  const cycleState = useCycleState();
  const { startFocusSession, startBreakSession, endSession } =
    useCycleManager();
  const [workScheduleInfo, setWorkScheduleInfo] =
    useState<WorkScheduleInfo | null>(null);
  const [showOverrideConfirm, setShowOverrideConfirm] = useState(false);
  const [workHoursStats, setWorkHoursStats] = useState<WorkHoursStats | null>(
    null
  );
  const [showStats, setShowStats] = useState(false);
  const [actionFeedback, setActionFeedback] = useState<string | null>(null);

  // Load work schedule info on mount
  useEffect(() => {
    const loadWorkScheduleInfo = async () => {
      try {
        const info = await CycleManager.getWorkScheduleInfo();
        setWorkScheduleInfo(info);
      } catch (error) {
        console.error("Failed to load work schedule info:", error);
      }
    };

    loadWorkScheduleInfo();
  }, []);

  // Load work hours stats
  const loadWorkHoursStats = async () => {
    try {
      const stats = await getWorkHoursStats(30);
      setWorkHoursStats(stats);
      setShowStats(true);
    } catch (error) {
      console.error("Failed to load work hours stats:", error);
    }
  };

  const handleStartFocus = async (override: boolean = false) => {
    try {
      setActionFeedback("Starting focus session...");
      await startFocusSession(override);
      setShowOverrideConfirm(false);
      setActionFeedback("Focus session started!");
      setTimeout(() => setActionFeedback(null), 2000);
    } catch (error) {
      console.error("Failed to start focus session:", error);
      setActionFeedback(null);
      // Show override option if outside work hours
      if (
        error instanceof Error &&
        error.message.includes("outside work hours")
      ) {
        setShowOverrideConfirm(true);
      }
    }
  };

  const handleStartBreak = async () => {
    try {
      setActionFeedback("Starting break...");
      await startBreakSession(false);
      setActionFeedback("Break started!");
      setTimeout(() => setActionFeedback(null), 2000);
    } catch (error) {
      console.error("Failed to start break:", error);
      setActionFeedback("Failed to start break");
      setTimeout(() => setActionFeedback(null), 3000);
    }
  };

  const handleEndSession = async () => {
    try {
      setActionFeedback("Ending session...");
      await endSession(false);
      setActionFeedback("Session ended!");
      setTimeout(() => setActionFeedback(null), 2000);
    } catch (error) {
      console.error("Failed to end session:", error);
      setActionFeedback("Failed to end session");
      setTimeout(() => setActionFeedback(null), 3000);
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
    <div className="bg-gray-200 opacity-95 rounded-lg p-6 border transition-all duration-300 hover:shadow-lg">
      {/* Action feedback toast */}
      {actionFeedback && (
        <div className="mb-4 p-3 bg-blue-500/20 border border-blue-500/30 rounded-lg text-blue-300 text-sm text-center animate-slide-in-down">
          {actionFeedback}
        </div>
      )}
      
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
              onClick={() => handleStartFocus(false)}
              disabled={!cycleState.can_start}
              className="flex-1 flex items-center justify-center space-x-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed text-white font-medium py-3 px-4 rounded-lg transition-all duration-200 transform hover:scale-105 active:scale-95"
            >
              <Play className="w-5 h-5" />
              <span>Start Focus</span>
            </button>
            <button
              onClick={handleStartBreak}
              className="flex-1 flex items-center justify-center space-x-2 bg-green-600 hover:bg-green-700 text-white font-medium py-3 px-4 rounded-lg transition-all duration-200 transform hover:scale-105 active:scale-95"
            >
              <Coffee className="w-5 h-5" />
              <span>Start Break</span>
            </button>
          </>
        )}

        {cycleState.phase !== "idle" && (
          <div className="flex w-full items-center justify-center">
            <button
              onClick={handleEndSession}
              className="w-fit flex items-center justify-center space-x-2 bg-red-600 hover:bg-red-700 text-white font-medium py-2 px-3 rounded-lg transition-all duration-200 transform hover:scale-105 active:scale-95"
            >
              <Square className="w-5 h-5" />
              <span>End Session</span>
            </button>
          </div>
        )}
      </div>

      {/* Work schedule information */}
      {workScheduleInfo && (
        <div className="mt-4 p-3 bg-gray-700/50 border border-gray-600 rounded-lg">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center space-x-2">
              <Clock className="w-4 h-4 text-blue-400" />
              <span className="text-sm font-medium text-gray-300">
                Work Schedule
              </span>
            </div>
            {workScheduleInfo.start_time && workScheduleInfo.end_time && (
              <button
                onClick={loadWorkHoursStats}
                className="text-xs text-blue-400 hover:text-blue-300 flex items-center space-x-1"
              >
                <TrendingUp className="w-3 h-3" />
                <span>View Stats</span>
              </button>
            )}
          </div>
          <div className="text-sm text-gray-400">
            {workScheduleInfo.start_time && workScheduleInfo.end_time ? (
              <>
                <p>
                  Hours: {workScheduleInfo.start_time} -{" "}
                  {workScheduleInfo.end_time}
                </p>
                <p className="mt-1">
                  Status:{" "}
                  <span
                    className={
                      workScheduleInfo.is_within_hours
                        ? "text-green-400"
                        : "text-yellow-400"
                    }
                  >
                    {workScheduleInfo.is_within_hours
                      ? "Within work hours"
                      : "Outside work hours"}
                  </span>
                </p>
              </>
            ) : (
              <p>No work schedule configured</p>
            )}
          </div>
        </div>
      )}

      {/* Work hours statistics */}
      {showStats && workHoursStats && (
        <div className="mt-4 p-4 bg-gray-700/50 border border-gray-600 rounded-lg">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-semibold text-gray-300">
              Work Hours Compliance (Last 30 Days)
            </h3>
            <button
              onClick={() => setShowStats(false)}
              className="text-xs text-gray-400 hover:text-gray-300"
            >
              Hide
            </button>
          </div>

          <div className="space-y-3">
            {/* Compliance percentage */}
            <div>
              <div className="flex items-center justify-between mb-1">
                <span className="text-xs text-gray-400">Compliance Rate</span>
                <span className="text-sm font-semibold text-green-400">
                  {workHoursStats.compliance_percentage.toFixed(1)}%
                </span>
              </div>
              <div className="w-full bg-gray-600 rounded-full h-2">
                <div
                  className="bg-green-500 h-2 rounded-full transition-all"
                  style={{
                    width: `${workHoursStats.compliance_percentage}%`,
                  }}
                />
              </div>
            </div>

            {/* Session counts */}
            <div className="grid grid-cols-2 gap-3">
              <div className="bg-gray-800/50 rounded p-2">
                <p className="text-xs text-gray-400 mb-1">Within Hours</p>
                <p className="text-lg font-semibold text-green-400">
                  {workHoursStats.within_work_hours}
                </p>
                <p className="text-xs text-gray-500">
                  {workHoursStats.total_focus_minutes_within} min
                </p>
              </div>
              <div className="bg-gray-800/50 rounded p-2">
                <p className="text-xs text-gray-400 mb-1">Outside Hours</p>
                <p className="text-lg font-semibold text-yellow-400">
                  {workHoursStats.outside_work_hours}
                </p>
                <p className="text-xs text-gray-500">
                  {workHoursStats.total_focus_minutes_outside} min
                </p>
              </div>
            </div>

            {/* Total sessions */}
            <div className="text-center pt-2 border-t border-gray-600">
              <p className="text-xs text-gray-400">Total Focus Sessions</p>
              <p className="text-xl font-bold text-white">
                {workHoursStats.total_sessions}
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Override confirmation dialog */}
      {showOverrideConfirm && (
        <div className="mt-4 p-4 bg-yellow-900/30 border border-yellow-700/50 rounded-lg">
          <div className="flex items-start space-x-3">
            <AlertCircle className="w-5 h-5 text-yellow-400 flex-shrink-0 mt-0.5" />
            <div className="flex-1">
              <h3 className="text-sm font-semibold text-yellow-400 mb-2">
                Outside Work Hours
              </h3>
              <p className="text-sm text-gray-300 mb-3">
                You're trying to start a focus session outside your configured
                work hours. Would you like to proceed anyway?
              </p>
              <div className="flex space-x-2">
                <button
                  onClick={() => handleStartFocus(true)}
                  className="flex-1 bg-yellow-600 hover:bg-yellow-700 text-white font-medium py-2 px-3 rounded-lg transition-colors text-sm"
                >
                  Yes, Start Anyway
                </button>
                <button
                  onClick={() => setShowOverrideConfirm(false)}
                  className="flex-1 bg-gray-600 hover:bg-gray-700 text-white font-medium py-2 px-3 rounded-lg transition-colors text-sm"
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
