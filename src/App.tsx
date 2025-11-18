import { HashRouter, Routes, Route } from "react-router-dom";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Login from "./pages/Login";
import Dashboard from "./pages/Dashboard";
import OnboardingWizard from "./components/OnboardingWizard";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { ToastContainer } from "./components/ToastContainer";
import { CycleSync } from "./components/CycleSync";
import { MenuBarPopover } from "./components/MenuBarPopover";
import { BreakTransitionWindow } from "./components/BreakTransitionWindow";
import { errorHandler } from "./lib/errorHandler";
import Stats from "./pages/Stats";
import Settings from "./pages/Settings";

export default function App() {
  const [needsOnboarding, setNeedsOnboarding] = useState<boolean | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [windowLabel, setWindowLabel] = useState<string>("");

  useEffect(() => {
    const checkAppState = async () => {
      try {
        // Get current window label
        const currentWindow = getCurrentWindow();
        const label = currentWindow.label;
        setWindowLabel(label);

        // If this is a special window (menu-bar-popover, break-transition, etc.), skip onboarding check
        if (label === "menu-bar-popover" || label === "break-transition") {
          setIsLoading(false);
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

  // Render MenuBarPopover for menu-bar-popover window
  if (windowLabel === "menu-bar-popover") {
    return (
      <ErrorBoundary>
        <CycleSync />
        <div className="min-h-screen flex items-start justify-center pt-2">
          <MenuBarPopover
            onClose={async () => {
              try {
                await invoke("hide_menu_bar_popover");
              } catch (error) {
                console.error("Failed to hide menu bar popover:", error);
              }
            }}
          />
        </div>
      </ErrorBoundary>
    );
  }

  // Render BreakTransitionWindow for break-transition window
  if (windowLabel === "break-transition") {
    return (
      <ErrorBoundary>
        <div className="min-h-screen bg-zinc-900 flex items-center justify-center">
          <BreakTransitionWindow />
        </div>
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
      <ToastContainer />
    </ErrorBoundary>
  );
}
