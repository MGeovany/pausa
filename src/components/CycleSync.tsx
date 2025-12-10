import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { useAppStore } from "../store";
import { CycleManager } from "../lib/cycleCommands";
import type { CycleEventData } from "../types";

/**
 * Global component to sync cycle state across all pages
 * This ensures the cycle timer continues running even when navigating between pages
 */
export function CycleSync() {
  const {
    setCycleState,
    showFocusWidget,
    hideFocusWidget,
    showBreakOverlay,
    hideBreakOverlay,
  } = useAppStore();

  // Keep main window centered/normalized to avoid OS repositioning
  const normalizeMainWindow = async () => {
    try {
      const win = getCurrentWindow();
      if (win.label !== "main") return;
      await win.setSize(new LogicalSize(1280, 800));
      await win.center();
    } catch (error) {
      console.error("Failed to normalize main window:", error);
    }
  };

  // Initialize cycle orchestrator on mount
  useEffect(() => {
    const initializeCycle = async () => {
      try {
        const state = await CycleManager.initialize();
        setCycleState(state);
      } catch (error) {
        console.error("Failed to initialize cycle manager:", error);
      }
    };

    initializeCycle();
  }, [setCycleState]);

  // Set up event listener for cycle events
  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      try {
        unlisten = await listen<CycleEventData>("cycle-event", (event) => {
          const cycleEvent = event.payload;
          const phaseLabel =
            "phase" in cycleEvent ? cycleEvent.phase : "n/a";
          console.log("📡 [Frontend] Cycle event received:", cycleEvent);
          console.log(
            "📡 [Frontend] Event type:",
            cycleEvent.type,
            "Phase:",
            phaseLabel
          );
          normalizeMainWindow();

          // Handle different event types
          switch (cycleEvent.type) {
            case "phase_started":
              console.log("🚀 [Frontend] Phase started:", cycleEvent.phase);
              // Immediately sync state from backend to ensure we have the latest state
              CycleManager.getState()
                .then((state) => {
                  console.log("✅ [Frontend] Synced state after phase_started:", state);
                  setCycleState(state);
                  // Update UI based on phase
                  if (cycleEvent.phase === "focus") {
                    console.log("🎯 [Frontend] Showing focus widget");
                    showFocusWidget();
                    hideBreakOverlay();
                    // Refresh stats when a new focus session starts to show updated progress
                    window.dispatchEvent(new CustomEvent('refresh-stats'));
                  } else if (
                    cycleEvent.phase === "short_break" ||
                    cycleEvent.phase === "long_break"
                  ) {
                    console.log("☕ [Frontend] Showing break overlay for:", cycleEvent.phase);
                    hideFocusWidget();
                    showBreakOverlay();
                  }
                })
                .catch((error) => {
                  console.error("Failed to sync state after phase_started:", error);
                  // Still update UI even if state sync fails
                  if (cycleEvent.phase === "focus") {
                    showFocusWidget();
                    hideBreakOverlay();
                    // Refresh stats even if state sync fails
                    window.dispatchEvent(new CustomEvent('refresh-stats'));
                  } else if (
                    cycleEvent.phase === "short_break" ||
                    cycleEvent.phase === "long_break"
                  ) {
                    hideFocusWidget();
                    showBreakOverlay();
                  }
                });
              break;

            case "phase_ended":
              console.log("🏁 [Frontend] Phase ended:", cycleEvent.phase);
              // Sync state when phase ends
              CycleManager.getState()
                .then((state) => {
                  console.log("✅ [Frontend] Synced state after phase_ended:", state);
                  setCycleState(state);
                  // Update UI based on ended phase
                  if (cycleEvent.phase === "focus") {
                    console.log("🎯 [Frontend] Hiding focus widget");
                    hideFocusWidget();
                    // Trigger stats refresh after focus ends
                    window.dispatchEvent(new CustomEvent('refresh-stats'));
                  } else if (
                    cycleEvent.phase === "short_break" ||
                    cycleEvent.phase === "long_break"
                  ) {
                    console.log("☕ [Frontend] Hiding break overlay");
                    hideBreakOverlay();
                    // Trigger stats refresh after break ends
                    window.dispatchEvent(new CustomEvent('refresh-stats'));
                  }
                })
                .catch((error) => {
                  console.error("Failed to sync state after phase_ended:", error);
                  // Still update UI even if state sync fails
                  if (cycleEvent.phase === "focus") {
                    hideFocusWidget();
                  } else if (
                    cycleEvent.phase === "short_break" ||
                    cycleEvent.phase === "long_break"
                  ) {
                    hideBreakOverlay();
                  }
                });
              break;

            case "pre_alert":
              console.log("⏰ Pre-alert: 2 minutes remaining");
              break;

            case "cycle_completed":
              console.log(`✅ Cycle ${cycleEvent.cycle_count} completed`);
              // Sync state when cycle completes to update cycle_count
              CycleManager.getState()
                .then((state) => {
                  console.log("✅ [Frontend] Synced state after cycle_completed:", state);
                  setCycleState(state);
                  // Refresh stats to show updated progress
                  window.dispatchEvent(new CustomEvent('refresh-stats'));
                })
                .catch((error) => {
                  console.error("Failed to sync state after cycle_completed:", error);
                });
              break;

            case "long_break_reached":
              console.log(
                `🎉 Long break reached after ${cycleEvent.cycles_completed} cycles`
              );
              break;
          }
        });
      } catch (error) {
        console.error("Failed to setup cycle event listener:", error);
      }
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [
    setCycleState,
    showFocusWidget,
    hideFocusWidget,
    showBreakOverlay,
    hideBreakOverlay,
  ]);

  // Set up timer to tick every second and sync state
  // This runs globally and persists across page navigations
  useEffect(() => {
    const mainTimer = setInterval(async () => {
      try {
        const state = await CycleManager.getState();

        // Tick if running OR if we're in an active phase (not idle)
        // This ensures smooth transitions when focus ends and break starts automatically
        if (state.is_running || state.phase !== "idle") {
          const updatedState = await CycleManager.tick();
          if (updatedState.phase !== state.phase) {
            console.log("🔄 [Frontend] Phase changed:", state.phase, "->", updatedState.phase);
          }
          setCycleState(updatedState);
        } else {
          // Just sync state if idle and not running
          setCycleState(state);
        }
      } catch (error) {
        console.error("❌ [Frontend] Failed to tick/sync cycle:", error);
      }
    }, 1000);

    return () => {
      clearInterval(mainTimer);
    };
  }, [setCycleState]);

  return null; // This component doesn't render anything
}
