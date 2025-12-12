import { EmergencyOverride } from "./EmergencyOverride";
import type { BreakSession, CycleState } from "../types";
import {
  BreakActivityChecklist,
  CountdownTimer,
  StrictModeBreakUI,
  CycleProgressIndicator,
} from "./break/BreakOverlayParts";
import { useBreakOverlayLogic } from "./break/useBreakOverlayLogic";

interface BreakOverlayProps {
  breakSession: BreakSession;
  onCompleteBreak: () => void;
  onEmergencyOverride: (pin: string) => Promise<boolean>;
  cycleState?: CycleState | null;
  userName?: string;
  isStrictMode?: boolean;
  emergencyKeyCombination?: string;
}

export function BreakOverlay({
  breakSession,
  onCompleteBreak,
  onEmergencyOverride,
  cycleState,
  userName,
  isStrictMode = false,
  emergencyKeyCombination,
}: BreakOverlayProps) {
  const {
    resolvedBreakType,
    remaining,
    clampedRemaining,
    activity,
    currentCycleState,
    settings,
    canCompleteBreak,
    showCompletionInterface,
    accentColor,
    bypassAttempts,
    showEmergencyModal,
    setShowEmergencyModal,
    handleChecklistUpdate,
    handleEmergencyOverride,
  } = useBreakOverlayLogic({
    breakSession,
    cycleState,
    isStrictMode,
    emergencyKeyCombination,
    onCompleteBreak,
    onEmergencyOverride,
  });

  console.log("🔍 [BreakOverlay] Render:", {
    isStrictMode,
    currentCycleState,
    cycleStateProp: cycleState,
    remaining,
  });

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{
        width: "100vw",
        height: "100vh",
        position: "fixed",
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: isStrictMode ? "#000000" : undefined,
      }}
    >
      {/* Main break content */}
      <div className="flex flex-col items-center justify-center min-h-screen p-8 w-full">
        {(() => {
          console.log("🔍 [BreakOverlay] Rendering content:", {
            isStrictMode,
            currentCycleState,
          });

          if (isStrictMode) {
            // Use cycleState directly - it's the source of truth from cycle_handler
            // This ensures we're always in sync with the backend
            console.log(
              "🔍 [BreakOverlay] Rendering StrictModeBreakUI with:",
              currentCycleState
            );
            return <StrictModeBreakUI cycleState={currentCycleState} />;
          }

          return (
            <>
              {/* Normal mode: Show full UI with activities */}
              {/* Break type indicator */}
              <div className="mb-8">
                <div
                  className={`inline-flex items-center px-4 py-2 bg-${accentColor}-500/20 rounded-full`}
                >
                  <div
                    className={`w-2 h-2 bg-${accentColor}-400 rounded-full mr-3 animate-pulse`}
                  ></div>
                  <span className={`text-${accentColor}-200 font-medium`}>
                    {resolvedBreakType === "short"
                      ? "☕ Short Break"
                      : "🌟 Long Break"}
                  </span>
                </div>
              </div>

              {/* Cycle progress indicator */}
              {currentCycleState && currentCycleState.cycle_count > 0 && (
                <CycleProgressIndicator
                  cycleCount={currentCycleState.cycle_count}
                  cyclesPerLongBreak={settings.cyclesPerLongBreak}
                  breakType={resolvedBreakType}
                />
              )}

              {/* Countdown timer */}
              <CountdownTimer
                remaining={clampedRemaining}
                breakType={resolvedBreakType}
                userName={userName}
              />

              {/* Break activity checklist */}
              <BreakActivityChecklist
                activity={activity}
                onChecklistUpdate={handleChecklistUpdate}
              />

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
          );
        })()}
      </div>

      {/* Emergency override modal - only show in normal mode */}
      {!isStrictMode && (
        <EmergencyOverride
          isOpen={showEmergencyModal}
          onClose={() => setShowEmergencyModal(false)}
          onOverride={handleEmergencyOverride}
          emergencyWindowSeconds={45}
        />
      )}
    </div>
  );
}
