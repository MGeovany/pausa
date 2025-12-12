import { HashRouter, Routes, Route } from "react-router-dom";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
import Login from "./pages/Login";
import Dashboard from "./pages/Dashboard";
import OnboardingWizard from "./components/OnboardingWizard";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { CycleSync } from "./components/CycleSync";
import { BreakOverlay } from "./components/BreakOverlay";
import { errorHandler } from "./lib/errorHandler";
import Stats from "./pages/Stats";
import Settings from "./pages/Settings";
import type { BreakSession, CycleState } from "./types";

export default function App() {
  const [needsOnboarding, setNeedsOnboarding] = useState<boolean | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [windowLabel, setWindowLabel] = useState<string>("");
  const [breakSession, setBreakSession] = useState<BreakSession | null>(null);
  const [isStrictMode, setIsStrictMode] = useState(false);
  const [emergencyKeyCombination, setEmergencyKeyCombination] = useState<
    string | undefined
  >(undefined);
  const [cycleState, setCycleState] = useState<CycleState | null>(null);

  useEffect(() => {
    const checkAppState = async () => {
      try {
        // Get current window label
        const currentWindow = getCurrentWindow();
        const label = currentWindow.label;
        setWindowLabel(label);

        // If this is a special window (break-overlay), skip onboarding check
        if (label === "break-overlay") {
          setIsLoading(false);

          // If break-overlay window, fetch break session and settings
          if (label === "break-overlay") {
            try {
              const currentBreak = await invoke<BreakSession | null>(
                "get_current_break"
              );
              console.log(
                "🖥️ [App] Break session loaded:",
                currentBreak ? `remaining:${currentBreak.remaining}` : "null"
              );
              setBreakSession(currentBreak);

              const settings = await invoke<any>("get_settings");
              const strictMode = settings.strictMode || false;
              console.log("🖥️ [App] Strict mode:", strictMode);
              setIsStrictMode(strictMode);
              setEmergencyKeyCombination(settings.emergencyKeyCombination);

              // Fetch cycle state for break-overlay window
              // This ensures StrictModeBreakUI has access to cycleState
              const { CycleManager } = await import("./lib/cycleCommands");
              try {
                const cycleState = await CycleManager.getState();
                console.log(
                  "🖥️ [App] Cycle state loaded for break-overlay:",
                  cycleState
                );
                setCycleState(cycleState);
              } catch (error) {
                console.error("❌ [App] Failed to fetch cycle state:", error);
              }
            } catch (error) {
              console.error(
                "❌ [App] Failed to fetch break session or settings:",
                error
              );
            }
          }

          return;
        }

        // Check if onboarding is needed (first launch or onboarding not complete)
        const firstLaunch = await invoke<boolean>("is_first_launch");
        const onboardingComplete = await invoke<boolean>(
          "get_onboarding_status"
        );

        // User needs onboarding if it's first launch OR onboarding is not complete
        const needsOnboardingFlow = firstLaunch || !onboardingComplete;
        setNeedsOnboarding(needsOnboardingFlow);

        console.log("App state check:", {
          firstLaunch,
          onboardingComplete,
          needsOnboardingFlow,
        });
      } catch (error) {
        console.error("Error checking app state:", error);
        errorHandler.logError(
          error as Error,
          "App.checkAppState",
          "Checking if onboarding is needed"
        );
        // Default to showing onboarding if we can't determine state (safer)
        setNeedsOnboarding(true);
      } finally {
        setIsLoading(false);
      }
    };

    checkAppState();
  }, []);

  // Force main window to a sane size/position on mount to avoid “esquina” issues
  useEffect(() => {
    if (windowLabel !== "main") return;

    const normalizeMainWindow = async () => {
      try {
        const win = getCurrentWindow();
        await win.setSize(new LogicalSize(1280, 800));
        await win.center();
      } catch (error) {
        console.error("Failed to normalize main window position/size:", error);
      }
    };

    normalizeMainWindow();
  }, [windowLabel]);

  // Sync cycle state for break-overlay window (similar to CycleSync but for this window)
  useEffect(() => {
    if (windowLabel !== "break-overlay") return;

    const syncCycleState = async () => {
      try {
        const { CycleManager } = await import("./lib/cycleCommands");
        const state = await CycleManager.getState();
        setCycleState(state);
      } catch (error) {
        console.error("❌ [App] Failed to sync cycle state:", error);
      }
    };

    // Initial sync
    syncCycleState();

    // Sync every second to keep cycleState updated
    const interval = setInterval(syncCycleState, 1000);

    return () => clearInterval(interval);
  }, [windowLabel]);

  // Listen for break updates to refresh breakSession when a new break starts
  useEffect(() => {
    if (windowLabel !== "break-overlay") return;

    let unlisten: (() => void) | null = null;

    const setupBreakListener = async () => {
      try {
        // Listen for cycle events to detect when a new break starts
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen<any>("cycle-event", async (event) => {
          const cycleEvent = event.payload;

          // When a break phase starts, refresh the break session
          if (
            cycleEvent.type === "phase_started" &&
            (cycleEvent.phase === "short_break" ||
              cycleEvent.phase === "long_break")
          ) {
            console.log(
              "🔄 [App] Break phase started, refreshing break session"
            );
            try {
              const currentBreak = await invoke<BreakSession | null>(
                "get_current_break"
              );
              if (currentBreak) {
                console.log(
                  "✅ [App] Break session refreshed:",
                  `remaining:${currentBreak.remaining}`
                );
                setBreakSession(currentBreak);
              }
            } catch (error) {
              console.error("❌ [App] Failed to refresh break session:", error);
            }
          }
        });
      } catch (error) {
        console.error("❌ [App] Failed to setup break listener:", error);
      }
    };

    setupBreakListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [windowLabel]);

  const handleOnboardingComplete = async (config: any) => {
    try {
      // Apply the onboarding configuration to settings
      await invoke("apply_onboarding_config_to_settings", { config });

      // Mark onboarding as no longer needed
      setNeedsOnboarding(false);

      console.log("Onboarding completed successfully");
    } catch (error) {
      console.error("Error completing onboarding:", error);
    }
  };

  const handleSkipOnboarding = () => {
    setNeedsOnboarding(false);
  };

  // Show loading state while checking app state
  if (isLoading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-zinc-900 via-zinc-800 to-zinc-900 flex items-center justify-center">
        <div className="text-white text-lg">Loading...</div>
      </div>
    );
  }

  // Render BreakOverlay for break-overlay window
  if (windowLabel === "break-overlay") {
    console.log(
      "🖥️ [App] BreakOverlay window - session:",
      breakSession
        ? `remaining:${breakSession.remaining} strict:${isStrictMode}`
        : "null"
    );

    // If no break session, immediately hide overlay window to avoid blank/black view
    if (!breakSession) {
      invoke("hide_fullscreen_break_overlay").catch((error) =>
        console.error("❌ [App] Failed to hide break overlay:", error)
      );
      return null;
    }

    return (
      <ErrorBoundary>
        <BreakOverlay
          breakSession={breakSession}
          cycleState={cycleState}
          onCompleteBreak={async () => {
            try {
              await invoke("hide_fullscreen_break_overlay");
            } catch (error) {
              console.error("❌ [App] Failed to hide break overlay:", error);
            }
          }}
          onEmergencyOverride={async (pin: string) => {
            try {
              return await invoke<boolean>("verify_emergency_pin", { pin });
            } catch (error) {
              console.error("❌ [App] Failed to verify emergency pin:", error);
              return false;
            }
          }}
          isStrictMode={isStrictMode}
          emergencyKeyCombination={emergencyKeyCombination}
        />
      </ErrorBoundary>
    );
  }

  // Show onboarding if needed
  if (needsOnboarding) {
    return (
      <OnboardingWizard
        onComplete={handleOnboardingComplete}
        onSkip={handleSkipOnboarding}
      />
    );
  }

  // Show normal app flow
  return (
    <ErrorBoundary>
      <CycleSync />
      <HashRouter>
        <Routes>
          <Route path="/" element={<Login />} />
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/stats" element={<Stats />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </HashRouter>
    </ErrorBoundary>
  );
}
