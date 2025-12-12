import { useMemo } from "react";
import { Play, Pause, Square } from "lucide-react";
import { CycleState, UserSettings, StrictModeState } from "../../types";

interface SessionTimerProps {
  cycleState: CycleState | null;
  strictModeState: StrictModeState | null;
  settings: UserSettings;
  onStartRoutine: () => Promise<void>;
  onPause: () => Promise<void>;
  onResume: () => Promise<void>;
  onEndSession: (completed: boolean) => Promise<void>;
}

export function SessionTimer({
  cycleState,
  strictModeState,
  settings,
  onStartRoutine,
  onPause,
  onResume,
  onEndSession,
}: SessionTimerProps) {
  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, "0")}:${secs
      .toString()
      .padStart(2, "0")}`;
  };

  const phaseLabel = useMemo(() => {
    if (!cycleState) return "Idle";
    switch (cycleState.phase) {
      case "focus":
        return "Focus";
      case "short_break":
        return "Short Break";
      case "long_break":
        return "Long Break";
      default:
        return "Idle";
    }
  }, [cycleState]);

  return (
    <section className="lg:col-span-2 bg-gray-900/40 border border-gray-800 rounded-xl p-6 md:p-8">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-sm font-semibold text-gray-300 flex items-center gap-2">
          Session
          {strictModeState?.isActive && (
            <span className="inline-flex items-center gap-1.5 text-xs text-amber-300 bg-amber-500/20 border border-amber-500/30 rounded-full px-3 py-1 font-medium animate-pulse-glow">
              <span className="w-1.5 h-1.5 bg-amber-400 rounded-full animate-pulse"></span>
              🔒 Strict Mode
            </span>
          )}
        </h2>
        <div className="text-xs text-gray-500">
          {cycleState ? phaseLabel : "Loading…"}
        </div>
      </div>

      <div className="flex flex-col items-center justify-center gap-4 py-6">
        <div className="text-center">
          <div className="text-5xl sm:text-6xl md:text-7xl font-mono font-semibold">
            TIME{" "}
            {cycleState
              ? cycleState.phase === "idle"
                ? "--:--"
                : formatTime(cycleState.remaining_seconds)
              : "--:--"}
          </div>
          <div className="text-xs text-gray-500 mt-2">
            {cycleState
              ? cycleState.is_running
                ? "Running"
                : cycleState.phase === "idle"
                ? "Idle"
                : "Paused"
              : "Syncing…"}
          </div>
        </div>

        <div className="flex items-center justify-center gap-2 flex-wrap">
          {cycleState?.phase === "idle" && (
            <>
              <button
                onClick={onStartRoutine}
                disabled={!cycleState?.can_start}
                className={`inline-flex items-center gap-2 text-gray-100 border rounded-lg px-4 py-2 text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-all ${
                  settings.strictMode
                    ? "bg-amber-600 hover:bg-amber-400 border-amber-500 shadow-lg shadow-amber-500/70"
                    : "bg-gray-800 hover:bg-gray-700 border-gray-800"
                }`}
              >
                <Play className="w-4 h-4" />
                Start Focus
              </button>
              {settings.strictMode && (
                <div className="flex items-center justify-center gap-1.5 text-xs text-amber-300 w-full text-center mt-2 animate-pulse">
                  <span className="w-1.5 h-1.5 bg-amber-400 rounded-full"></span>
                  Strict mode will minimize to menu bar
                </div>
              )}
            </>
          )}

          {cycleState && cycleState.phase !== "idle" && (
            <>
              {!cycleState.is_running ? (
                <button
                  onClick={onResume}
                  className="inline-flex items-center gap-2 bg-gray-800 hover:bg-gray-700 text-gray-100 border border-gray-800 rounded-lg px-4 py-2 text-sm font-medium"
                >
                  <Play className="w-4 h-4" />
                  Resume
                </button>
              ) : (
                <button
                  onClick={onPause}
                  className="inline-flex items-center gap-2 bg-gray-900 hover:bg-gray-800 border border-gray-800 rounded-lg px-4 py-2 text-sm"
                >
                  <Pause className="w-4 h-4" />
                  Pause
                </button>
              )}
              <button
                onClick={() => onEndSession(false)}
                className="inline-flex items-center gap-2 bg-gray-900 hover:bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-sm text-red-300 hover:text-red-200"
              >
                <Square className="w-4 h-4" />
                End
              </button>
            </>
          )}
        </div>
      </div>
    </section>
  );
}
