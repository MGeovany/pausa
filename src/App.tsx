import { useEffect } from "react";
import { TauriCommandTest } from "./components/TauriCommandTest";
import { FocusWidget, useFocusWidget } from "./components/FocusWidget";
import { CommandPalette } from "./components/CommandPalette";
import { BreakOverlay } from "./components/BreakOverlay";
import { Settings } from "./components/Settings";
import { useAppStore, useUIState, useCurrentBreak } from "./store";
import { useCommands } from "./lib/commands";
import { tauriCommands } from "./lib/tauri";

function App() {
  const { session, onToggleSession, onResetSession, onOpenMenu } =
    useFocusWidget();
  const {
    isCommandPaletteOpen,
    isFocusWidgetVisible,
    isBreakOverlayVisible,
    isSettingsOpen,
  } = useUIState();
  const currentBreak = useCurrentBreak();
  const { toggleCommandPalette, hideBreakOverlay } = useAppStore();
  const { getAllCommands } = useCommands();

  // Break overlay handlers
  const handleCompleteBreak = async () => {
    try {
      await tauriCommands.completeBreak();
      hideBreakOverlay();
    } catch (error) {
      console.error("Failed to complete break:", error);
    }
  };

  const handleEmergencyOverride = async (pin: string): Promise<boolean> => {
    try {
      const success = await tauriCommands.verifyEmergencyPin(pin);
      if (success) {
        // Log the emergency override attempt
        console.log("Emergency override successful - logged for security");
        // Temporarily hide the break overlay for the emergency window
        // The break will resume after the emergency window expires
        return true;
      }
      return false;
    } catch (error) {
      console.error("Failed to verify emergency PIN:", error);
      return false;
    }
  };

  // Global hotkey handling
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Command/Ctrl + Space for command palette (only if break overlay is not visible)
      if (
        (event.metaKey || event.ctrlKey) &&
        event.code === "Space" &&
        !isBreakOverlayVisible
      ) {
        event.preventDefault();
        toggleCommandPalette();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [toggleCommandPalette, isBreakOverlayVisible]);

  // Sync window visibility with React state
  useEffect(() => {
    const checkWindowVisibility = async () => {
      try {
        const visible = await tauriCommands.isWindowVisible("settings");
        if (visible !== isSettingsOpen) {
          toggleSettings();
        }
      } catch (error) {
        console.error("Failed to check window visibility:", error);
      }
    };

    // Check on mount
    checkWindowVisibility();

    // Listen for window visibility changes
    const interval = setInterval(checkWindowVisibility, 500);
    return () => clearInterval(interval);
  }, [isSettingsOpen, toggleSettings]);

  return (
    <main className="bg-white min-h-screen">
      <p>Pausa Onboarding</p>
      {/*  <TauriCommandTest />

      {/* Focus Widget - only show when visible and session exists
      {isFocusWidgetVisible && session && (
        <FocusWidget
          session={session}
          onToggleSession={onToggleSession}
          onResetSession={onResetSession}
          onOpenMenu={onOpenMenu}
        />
      )}

      {/* Command Palette - only show when break overlay is not visible 
      {!isBreakOverlayVisible && (
        <CommandPalette
          isOpen={isCommandPaletteOpen}
          onClose={() => toggleCommandPalette()}
          commands={getAllCommands()}
        />
      )}

      {/* Break Overlay - fullscreen overlay for breaks 
      {isBreakOverlayVisible && currentBreak && (
        <BreakOverlay
          breakSession={currentBreak}
          onCompleteBreak={handleCompleteBreak}
          onEmergencyOverride={handleEmergencyOverride}
        />
      )}

//  Settings - settings window 
      {isSettingsOpen && <Settings />} */}
    </main>
  );
}

export default App;
