import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
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
          console.log("📡 Cycle event received:", cycleEvent);

          // Handle different event types
          switch (cycleEvent.type) {
            case "phase_started":
              if (cycleEvent.phase === "focus") {
                showFocusWidget();
                hideBreakOverlay();
              } else if (
                cycleEvent.phase === "short_break" ||
                cycleEvent.phase === "long_break"
              ) {
                hideFocusWidget();
                showBreakOverlay();
              }
              break;

            case "phase_ended":
              if (cycleEvent.phase === "focus") {
                hideFocusWidget();
              } else if (
                cycleEvent.phase === "short_break" ||
                cycleEvent.phase === "long_break"
              ) {
                hideBreakOverlay();
              }
              break;

            case "pre_alert":
              console.log("⏰ Pre-alert: 2 minutes remaining");
              break;

            case "cycle_completed":
              console.log(`✅ Cycle ${cycleEvent.cycle_count} completed`);
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

        // Only tick if running, otherwise just update state
        if (state.is_running) {
          const updatedState = await CycleManager.tick();
          setCycleState(updatedState);
        } else {
          setCycleState(state);
        }
      } catch (error) {
        console.error("Failed to tick/sync cycle:", error);
      }
    }, 1000);

    return () => {
      clearInterval(mainTimer);
    };
  }, [setCycleState]);

  return null; // This component doesn't render anything
}

