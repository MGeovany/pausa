import { useEffect } from "react";
import { useCycleManager } from "../lib/useCycleManager";
import { useCycleState, useAppStore, useStrictModeState } from "../store";
import { tauriCommands } from "../lib/tauri";
import { CycleManager } from "../lib/cycleCommands";
import {
  Sidebar,
  DashboardHeader,
  SessionTimer,
  ProgressCard,
  ShortcutsCard,
} from "../components/dashboard";

export default function Dashboard() {
  const cycleState = useCycleState();
  const strictModeState = useStrictModeState();
  const { setCycleState, updateSettings, settings } = useAppStore();
  const { startRoutine, pauseCycle, resumeCycle, endSession, resetCycleCount } =
    useCycleManager();

  // Load settings from database on mount
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const settings = await tauriCommands.getSettings();
        // Tauri automatically converts snake_case to camelCase
        updateSettings({
          focusDuration: settings.focusDuration,
          shortBreakDuration: settings.shortBreakDuration,
          longBreakDuration: settings.longBreakDuration,
          cyclesPerLongBreak: settings.cyclesPerLongBreak,
          preAlertSeconds: settings.preAlertSeconds,
          strictMode: settings.strictMode,
          pinHash: settings.pinHash,
          emergencyKeyCombination: settings.emergencyKeyCombination,
          blockedApps: settings.blockedApps || [],
          blockedWebsites: settings.blockedWebsites || [],
        });
      } catch (error) {
        console.error("Failed to load settings:", error);
        // Continue with default settings if loading fails
      }
    };

    loadSettings();
  }, [updateSettings]);

  // Sync cycle state when Dashboard mounts to ensure we have the latest state
  useEffect(() => {
    const syncCycleState = async () => {
      try {
        const state = await CycleManager.getState();
        setCycleState(state);
      } catch (error) {
        console.error("Failed to sync cycle state:", error);
      }
    };

    syncCycleState();
  }, [setCycleState]);

  // Handle starting routine with strict mode support
  const handleStartRoutine = async () => {
    try {
      // Note: StrictModeOrchestrator is automatically initialized and activated
      // in initialize_cycle_orchestrator when strict mode is enabled in settings.
      // No need to call activateStrictMode() manually here.

      // Start the focus session
      await startRoutine();
    } catch (error) {
      console.error("❌ [Dashboard] Failed to start routine:", error);
    }
  };

  return (
    <div className="min-h-screen bg-gray-950 text-gray-200">
      <div className="flex">
        <Sidebar />

        <main className="flex-1">
          <DashboardHeader />

          <div className="mx-auto w-full max-w-5xl px-6 py-4">
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
              <SessionTimer
                cycleState={cycleState}
                strictModeState={strictModeState}
                settings={settings}
                onStartRoutine={handleStartRoutine}
                onPause={pauseCycle}
                onResume={resumeCycle}
                onEndSession={endSession}
              />

              <ProgressCard
                cycleState={cycleState}
                settings={settings}
                onResetCycleCount={resetCycleCount}
              />
            </div>

            <div className="mt-6 grid grid-cols-1 lg:grid-cols-3 gap-6">
              <ShortcutsCard />
            </div>
          </div>
        </main>
      </div>
    </div>
  );
}
