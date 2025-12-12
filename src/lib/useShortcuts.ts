import { useEffect } from "react";
import { useCycleManager } from "./useCycleManager";
import { useAppStore } from "../store";
import { useNavigate } from "react-router-dom";

interface UseShortcutsOptions {
  enabled?: boolean;
  onToggleCommandPalette?: () => void;
}

export function useShortcuts(options: UseShortcutsOptions = {}) {
  const { enabled = true, onToggleCommandPalette } = options;
  const { startRoutine, pauseCycle, resumeCycle, endSession } =
    useCycleManager();
  const { cycleState, toggleCommandPalette } = useAppStore();
  const navigate = useNavigate();

  useEffect(() => {
    if (!enabled) return;

    const handleKeyDown = async (event: KeyboardEvent) => {
      // Don't handle shortcuts when typing in inputs
      const target = event.target as HTMLElement;
      if (
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable
      ) {
        // Allow Escape to work even in inputs
        if (event.key !== "Escape") {
          return;
        }
      }

      const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
      const cmdOrCtrl = isMac ? event.metaKey : event.ctrlKey;
      const shiftKey = event.shiftKey;

      // Command Palette: ⌘K (macOS) / Ctrl+K (Windows/Linux)
      if (cmdOrCtrl && (event.key === "k" || event.key === "K")) {
        event.preventDefault();
        if (onToggleCommandPalette) {
          onToggleCommandPalette();
        } else {
          toggleCommandPalette();
        }
        return;
      }

      // Start/Pause: Space (only when timer is active)
      if (
        event.key === " " &&
        !cmdOrCtrl &&
        !shiftKey &&
        cycleState &&
        cycleState.phase !== "idle"
      ) {
        event.preventDefault();
        if (cycleState.is_running) {
          await pauseCycle();
        } else {
          await resumeCycle();
        }
        return;
      }

      // Start Focus: ⌘⇧F (macOS) / Ctrl+Shift+F (Windows/Linux)
      if (cmdOrCtrl && shiftKey && event.key === "F") {
        event.preventDefault();
        if (cycleState?.phase === "idle") {
          await startRoutine();
        }
        return;
      }

      // End Session: Escape or ⌘⇧E (macOS) / Ctrl+Shift+E (Windows/Linux)
      if (
        event.key === "Escape" ||
        (cmdOrCtrl && shiftKey && event.key === "E")
      ) {
        event.preventDefault();
        if (cycleState && cycleState.phase !== "idle") {
          await endSession(false);
        }
        return;
      }

      // Navigate to Settings: ⌘, (macOS) / Ctrl+, (Windows/Linux)
      if (cmdOrCtrl && event.key === ",") {
        event.preventDefault();
        navigate("/settings");
        return;
      }

      // Navigate to Stats: ⌘⇧S (macOS) / Ctrl+Shift+S (Windows/Linux)
      if (cmdOrCtrl && shiftKey && event.key === "S") {
        event.preventDefault();
        navigate("/stats");
        return;
      }

      // Navigate to Dashboard: ⌘D (macOS) / Ctrl+D (Windows/Linux)
      if (cmdOrCtrl && event.key === "d" && !shiftKey) {
        event.preventDefault();
        navigate("/");
        return;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [
    enabled,
    cycleState,
    startRoutine,
    pauseCycle,
    resumeCycle,
    endSession,
    onToggleCommandPalette,
    toggleCommandPalette,
    navigate,
  ]);
}
