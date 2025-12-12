import { useState } from "react";
import type { BreakActivity } from "../../types";

export function CycleProgressIndicator({
  cycleCount,
  cyclesPerLongBreak,
  breakType,
}: {
  cycleCount: number;
  cyclesPerLongBreak: number;
  breakType: "short" | "long";
}) {
  const completedCycles =
    breakType === "long" ? cyclesPerLongBreak : cycleCount % cyclesPerLongBreak;

  return (
    <div className="mb-6">
      <div className="flex items-center justify-center space-x-2">
        <span className="text-sm text-gray-400 mr-2">Cycles completed:</span>
        {Array.from({ length: cyclesPerLongBreak }).map((_, index) => (
          <div
            key={index}
            className={`w-3 h-3 rounded-full transition-all duration-300 ${
              index < completedCycles
                ? breakType === "long"
                  ? "bg-amber-400 shadow-lg shadow-amber-400/50"
                  : "bg-blue-400 shadow-lg shadow-blue-400/50"
                : "bg-gray-600"
            }`}
          />
        ))}
        <span className="text-sm text-gray-400 ml-2">
          {completedCycles}/{cyclesPerLongBreak}
        </span>
      </div>
      {breakType === "long" && (
        <div className="text-center mt-2">
          <span className="text-amber-300 text-sm font-medium">
            🎉 Long break earned!
          </span>
        </div>
      )}
    </div>
  );
}

export function BreakActivityChecklist({
  activity,
  onChecklistUpdate,
}: {
  activity: BreakActivity;
  onChecklistUpdate: (completedItems: boolean[]) => void;
}) {
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
      <h3 className="text-2xl font-semibold text-white mb-2">
        {activity.title}
      </h3>
      <p className="text-gray-300 mb-6 leading-relaxed">
        {activity.description}
      </p>

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
                    ? "bg-blue-500 border-blue-500"
                    : "border-gray-400 group-hover:border-blue-400"
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
                  ? "text-gray-400 line-through"
                  : "text-white group-hover:text-blue-200"
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

export function CountdownTimer({
  remaining,
  breakType,
  userName,
}: {
  remaining: number;
  breakType: "short" | "long";
  userName?: string;
}) {
  const safeRemaining = Math.max(remaining, 0);
  const minutes = Math.floor(safeRemaining / 60);
  const seconds = safeRemaining % 60;

  const getMessage = () => {
    if (breakType === "long") {
      if (safeRemaining > 600)
        return userName
          ? `Great work, ${userName}. Take a proper rest`
          : "Take a proper rest";
      if (safeRemaining > 60) return "Enjoy your long break";
      return "Long break ending soon";
    } else {
      if (safeRemaining > 60) return "Take your time to recharge";
      return "Break ending soon";
    }
  };

  return (
    <div className="text-center mb-8">
      <div
        className={`font-light text-white mb-4 font-mono tracking-wider ${
          breakType === "long" ? "text-9xl" : "text-8xl"
        }`}
      >
        {minutes.toString().padStart(2, "0")}:
        {seconds.toString().padStart(2, "0")}
      </div>
      <div className="text-xl text-gray-300">{getMessage()}</div>
    </div>
  );
}

import { CycleState } from "../../types";

export function StrictModeBreakUI({
  cycleState,
}: {
  cycleState: CycleState | null;
}) {
  // Get remaining seconds directly from cycle state (source of truth)
  // This ensures we're always in sync with the backend cycle_handler

  console.log("CYCLE STATE", cycleState);
  const remaining = cycleState?.remaining_seconds ?? 0;
  const safeRemaining = Math.max(remaining, 0);
  const minutes = Math.floor(safeRemaining / 60);
  const seconds = safeRemaining % 60;

  return (
    <div className="flex flex-col items-center justify-center text-center gap-10">
      <div className="text-8xl font-bbh font-normal text-blue-100 mb-4">
        PAUSA
      </div>
      <div className="text-9xl font-semibold text-gray-200 font-mono">
        {minutes.toString().padStart(2, "0")}:
        {seconds.toString().padStart(2, "0")}
      </div>

      <div className="flex items-center justify-center my-8">
        <div className="relative w-[200px] h-[200px]">
          <div
            className="absolute inset-0 rounded-full animate-breathing-circle"
            style={{
              background:
                "radial-gradient(circle at 35% 30%, rgba(130,180,255,0.4), rgba(35,69,122,0.25) 45%, rgba(12,26,53,0.45) 70%, rgba(5,12,25,0.7))",
              boxShadow:
                "0 20px 50px rgba(7, 17, 35, 0.55), 0 0 35px rgba(65, 105, 225, 0.18)",
            }}
          />
          <div
            className="absolute inset-0 rounded-full pointer-events-none"
            style={{
              background:
                "radial-gradient(circle at 70% 60%, rgba(255,255,255,0.25), transparent 40%)",
              filter: "blur(14px)",
              opacity: 0.75,
            }}
          />
        </div>
      </div>
    </div>
  );
}
