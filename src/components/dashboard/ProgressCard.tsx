import { useMemo } from "react";
import { RefreshCcw, Target, Coffee, TrendingUp } from "lucide-react";
import { CycleState, UserSettings } from "../../types";

interface ProgressCardProps {
  cycleState: CycleState | null;
  settings: UserSettings;
  onResetCycleCount: () => Promise<void>;
}

export function ProgressCard({
  cycleState,
  settings,
  onResetCycleCount,
}: ProgressCardProps) {
  const cyclesPerLongBreak = settings.cyclesPerLongBreak || 4;
  const cyclesCompleted = cycleState?.cycle_count || 0;

  // Current cycle in the group (1-based) accounting for current phase
  const focusSessionsInGroup =
    cyclesCompleted + (cycleState?.phase === "focus" ? 1 : 0);
  const cycleInGroup = Math.max(
    1,
    ((focusSessionsInGroup - 1) % cyclesPerLongBreak) + 1
  );

  // Generate steps: Focus -> Break -> Focus -> Break -> ... -> Long Break
  const steps = useMemo(() => {
    const stepArray: Array<{
      type: "focus" | "break" | "long_break";
      index: number;
      completed: boolean;
    }> = [];
    for (let i = 0; i < cyclesPerLongBreak; i++) {
      stepArray.push({
        type: "focus",
        index: i + 1,
        completed: cyclesCompleted > i,
      });
      if (i < cyclesPerLongBreak - 1) {
        stepArray.push({
          type: "break",
          index: i + 1,
          completed: cyclesCompleted > i,
        });
      } else {
        stepArray.push({
          type: "long_break",
          index: i + 1,
          completed: cyclesCompleted > i,
        });
      }
    }
    return stepArray;
  }, [cyclesPerLongBreak, cyclesCompleted]);

  // Current step index (0-based) based on current phase
  const currentStepIndex = useMemo(() => {
    if (cycleState?.phase === "focus") {
      return Math.min(cyclesCompleted * 2, steps.length - 1);
    } else if (cycleState?.phase === "short_break") {
      return Math.min(Math.max(0, cyclesCompleted * 2 - 1), steps.length - 1);
    } else if (cycleState?.phase === "long_break") {
      return steps.length - 1;
    } else {
      // idle: if we've completed a full group (long break done), show "Cycle Complete!"
      if (cyclesCompleted > 0 && cyclesCompleted % cyclesPerLongBreak === 0) {
        return steps.length; // sentinel to trigger "Cycle Complete!"
      } else {
        // otherwise show next focus position
        return Math.min(cyclesCompleted * 2, steps.length - 1);
      }
    }
  }, [cycleState?.phase, cyclesCompleted, cyclesPerLongBreak, steps.length]);

  const isReadyForLongBreak = cycleInGroup === cyclesPerLongBreak;

  const nowPhase = cycleState?.phase || "idle";
  const nowType =
    nowPhase === "focus"
      ? "focus"
      : nowPhase === "short_break"
      ? "break"
      : nowPhase === "long_break"
      ? "long_break"
      : null;

  let nextType: "focus" | "break" | "long_break" | null = null;
  if (nowPhase === "focus") {
    nextType =
      (cyclesCompleted + 1) % cyclesPerLongBreak === 0 ? "long_break" : "break";
  } else if (nowPhase === "short_break") {
    nextType = "focus";
  } else if (nowPhase === "idle") {
    nextType =
      cyclesCompleted > 0 && cyclesCompleted % cyclesPerLongBreak === 0
        ? null
        : "focus";
  }

  const labelFor = (t: "focus" | "break" | "long_break") =>
    t === "focus" ? "Focus" : t === "long_break" ? "Long Break" : "Short Break";
  const minutesFor = (t: "focus" | "break" | "long_break") =>
    t === "focus"
      ? settings.focusDuration
      : t === "long_break"
      ? settings.longBreakDuration
      : settings.shortBreakDuration;
  const pillFor = (t: "focus" | "break" | "long_break") =>
    t === "focus"
      ? "bg-blue-500/10 text-blue-300 border-blue-400"
      : t === "long_break"
      ? "bg-amber-500/10 text-amber-300 border-amber-400"
      : "bg-green-500/10 text-green-300 border-green-400";

  const NowIcon =
    nowType === "focus" ? (
      <Target className="w-3 h-3 sm:w-3.5 sm:h-3.5" />
    ) : (
      <Coffee className="w-3 h-3 sm:w-3.5 sm:h-3.5" />
    );
  const NextIcon =
    nextType === "focus" ? (
      <Target className="w-3 h-3 sm:w-3.5 sm:h-3.5" />
    ) : (
      <Coffee className="w-3 h-3 sm:w-3.5 sm:h-3.5" />
    );

  const stepDisplay = Math.min(currentStepIndex + 1, steps.length);

  return (
    <section className="bg-gray-900/40 border border-gray-800 rounded-xl p-5">
      <h2 className="text-sm font-semibold text-gray-300 mb-4 flex items-center gap-2">
        Today's Progress
      </h2>
      <div className="grid grid-cols-2 gap-3 auto-rows-fr">
        <div className="rounded-lg border border-gray-800 bg-gray-900/60 p-4 sm:p-5 md:p-6 col-span-2">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-2">
              <TrendingUp className="w-4 h-4 text-amber-400" />
              <div className="text-[10px] uppercase tracking-wide text-gray-500">
                Cycle Progress
              </div>
            </div>
            <button
              onClick={onResetCycleCount}
              className="inline-flex items-center gap-1.5 rounded-lg border border-gray-800 bg-gray-900/60 px-2.5 py-1.5 text-[11px] text-gray-300 hover:text-white hover:bg-gray-900 transition-colors"
              title="Reset day"
            >
              <RefreshCcw className="w-3.5 h-3.5" />
              Reset day
            </button>
          </div>

          {/* Status row: Now / Next / Step */}
          <div className="flex flex-col sm:flex-row sm:flex-wrap items-start sm:items-center gap-2 sm:gap-2 mb-3">
            <div className="flex flex-wrap items-center gap-2">
              {nowType && (
                <span
                  className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs sm:text-sm ${pillFor(
                    nowType
                  )}`}
                >
                  {NowIcon}
                  <span className="hidden sm:inline">Now: </span>
                  {labelFor(nowType)}
                </span>
              )}
              {nextType ? (
                <span
                  className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs sm:text-sm ${pillFor(
                    nextType
                  )}`}
                  title={`${minutesFor(nextType)} minutes`}
                >
                  {NextIcon}
                  <span className="hidden sm:inline">Next: </span>
                  {labelFor(nextType)}
                  <span className="ml-1 text-[10px] sm:text-[11px] text-gray-400">
                    ({minutesFor(nextType)}m)
                  </span>
                </span>
              ) : (
                <span className="text-xs sm:text-sm text-gray-500">
                  No more steps
                </span>
              )}
            </div>
            <span className="text-[10px] sm:text-[11px] text-gray-500 sm:ml-auto">
              Step {stepDisplay} of {steps.length}
            </span>
          </div>

          <div className="mb-4">
            <div className="flex items-center justify-between mb-2 flex-wrap gap-2">
              <span className="text-sm sm:text-base font-semibold text-white">
                Cycle {cycleInGroup} of {cyclesPerLongBreak}
              </span>
            </div>

            {/* Visual steps progress */}
            <div className="overflow-x-auto pb-1 [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden">
              <div className="flex items-center gap-1.5 mb-3 min-w-max">
                {steps.map((step, idx) => {
                  const isCurrent = idx === currentStepIndex;
                  const isPast = idx < currentStepIndex;

                  let bgColor = "bg-gray-800";
                  let borderColor = "border-gray-700";
                  let icon = null;

                  if (step.type === "focus") {
                    bgColor = isPast
                      ? "bg-blue-600"
                      : isCurrent
                      ? "bg-blue-500"
                      : "bg-gray-800";
                    borderColor = isCurrent
                      ? "border-blue-400"
                      : "border-gray-700";
                    icon = <Target className="w-3 h-3" />;
                  } else if (step.type === "long_break") {
                    bgColor = isPast
                      ? "bg-amber-600"
                      : isCurrent
                      ? "bg-amber-500"
                      : "bg-gray-800";
                    borderColor = isCurrent
                      ? "border-amber-400"
                      : "border-gray-700";
                    icon = <Coffee className="w-3 h-3" />;
                  } else {
                    bgColor = isPast
                      ? "bg-green-600"
                      : isCurrent
                      ? "bg-green-500"
                      : "bg-gray-800";
                    borderColor = isCurrent
                      ? "border-green-400"
                      : "border-gray-700";
                    icon = <Coffee className="w-3 h-3" />;
                  }

                  return (
                    <div key={idx} className="flex items-center gap-1.5">
                      <div
                        className={`w-7 h-7 sm:w-8 sm:h-8 rounded-lg ${bgColor} border-2 ${borderColor} flex items-center justify-center text-white transition-all ${
                          isCurrent ? "scale-110 shadow-lg" : ""
                        }`}
                      >
                        {icon}
                      </div>
                      {idx < steps.length - 1 && (
                        <div
                          className={`w-3 sm:w-4 h-0.5 ${
                            isPast ? "bg-green-500" : "bg-gray-700"
                          }`}
                        />
                      )}
                    </div>
                  );
                })}
              </div>
            </div>

            {/* Current step label */}
            <div className="text-xs sm:text-sm text-gray-400 text-center sm:text-left">
              {currentStepIndex < steps.length ? (
                <>
                  <span className="hidden sm:inline">Current: </span>
                  <span className="text-white font-semibold">
                    {steps[currentStepIndex].type === "focus"
                      ? "Focus Session"
                      : steps[currentStepIndex].type === "long_break"
                      ? "Long Break"
                      : "Short Break"}
                  </span>
                </>
              ) : (
                <span className="text-green-400 font-semibold">
                  Productive day completed
                </span>
              )}
            </div>
          </div>

          {isReadyForLongBreak && cycleState?.phase !== "idle" && (
            <div className="mt-2 text-xs text-amber-400 font-semibold text-center">
              ⭐ Ready for long break!
            </div>
          )}

          {cycleState?.phase === "idle" &&
            cyclesCompleted > 0 &&
            cyclesCompleted % cyclesPerLongBreak === 0 && (
              <div className="mt-2 text-xs sm:text-sm text-green-400 font-semibold text-center">
                🎉 Productive day completed
              </div>
            )}
        </div>
      </div>
    </section>
  );
}
