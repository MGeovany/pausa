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
import type { BreakSession } from "./types";

export default function App() {
  const [needsOnboarding, setNeedsOnboarding] = useState<boolean | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [windowLabel, setWindowLabel] = useState<string>("");
  const [breakSession, setBreakSession] = useState<BreakSession | null>(null);
  const [isStrictMode, setIsStrictMode] = useState(false);
  const [emergencyKeyCombination, setEmergencyKeyCombination] = useState<
    string | undefined
  >(undefined);

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
              setBreakSession(currentBreak);

              const settings = await invoke<any>("get_settings");
              setIsStrictMode(settings.strictMode || false);
              setEmergencyKeyCombination(settings.emergencyKeyCombination);
            } catch (error) {
              console.error(
                "Failed to fetch break session or settings:",
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
    console.log("🖥️ [App] Rendering BreakOverlay component");
    console.log(
      "🖥️ [App] Break session state:",
      breakSession ? "loaded" : "not loaded"
    );
    console.log("🖥️ [App] Strict mode:", isStrictMode);

    if (breakSession) {
      console.log("✅ [App] Rendering BreakOverlay with break session:", {
        id: breakSession.id,
        type: breakSession.type,
        remaining: breakSession.remaining,
      });
    } else {
      console.log("⚠️ [App] Break session not yet available");
    }

    // If no break session, immediately hide overlay window to avoid blank/black view
    if (!breakSession) {
      invoke("hide_fullscreen_break_overlay").catch((error) =>
        console.error(
          "❌ [App] Failed to hide break overlay without session:",
          error
        )
      );
      return null;
    }

    return (
      <ErrorBoundary>
        <BreakOverlay
          breakSession={breakSession}
          onCompleteBreak={async () => {
            console.log("🖥️ [App] onCompleteBreak called");
            try {
              console.log("🖥️ [App] Calling hide_fullscreen_break_overlay...");
              await invoke("hide_fullscreen_break_overlay");
              console.log("✅ [App] Break overlay hidden successfully");
            } catch (error) {
              console.error("❌ [App] Failed to hide break overlay:", error);
            }
          }}
          onEmergencyOverride={async (pin: string) => {
            console.log("🖥️ [App] onEmergencyOverride called with pin");
            try {
              const result = await invoke<boolean>("verify_emergency_pin", {
                pin,
              });
              console.log(
                "🖥️ [App] Emergency pin verification result:",
                result
              );
              return result;
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
