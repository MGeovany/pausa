import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useAppStore } from "../store";
import { CycleManager } from "./cycleCommands";
import type { CycleEventData } from "../types";

/**
 * Hook to manage cycle state and handle real-time updates
 */
export const useCycleManager = () => {
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
                // Focus ended, prepare for break
                hideFocusWidget();
              } else if (
                cycleEvent.phase === "short_break" ||
                cycleEvent.phase === "long_break"
              ) {
                // Break ended
                hideBreakOverlay();
              }
              break;

            case "pre_alert":
              // Show pre-alert notification
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
  useEffect(() => {
    // Main timer that ticks every second
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

  return {
    startFocusSession: async () => {
      try {
        const state = await CycleManager.startFocusSession();
        setCycleState(state);
      } catch (error) {
        console.error("Failed to start focus session:", error);
        throw error;
      }
    },
    startBreakSession: async (forceLong?: boolean) => {
      try {
        const state = await CycleManager.startBreakSession(forceLong);
        setCycleState(state);
      } catch (error) {
        console.error("Failed to start break session:", error);
        throw error;
      }
    },
    pauseCycle: async () => {
      try {
        const state = await CycleManager.pause();
        setCycleState(state);
      } catch (error) {
        console.error("Failed to pause cycle:", error);
        throw error;
      }
    },
    resumeCycle: async () => {
      try {
        const state = await CycleManager.resume();
        setCycleState(state);
      } catch (error) {
        console.error("Failed to resume cycle:", error);
        throw error;
      }
    },
    endSession: async (completed: boolean) => {
      try {
        const state = await CycleManager.endSession(completed);
        setCycleState(state);
      } catch (error) {
        console.error("Failed to end session:", error);
        throw error;
      }
    },
    resetCycleCount: async () => {
      try {
        const state = await CycleManager.resetCycleCount();
        setCycleState(state);
      } catch (error) {
        console.error("Failed to reset cycle count:", error);
        throw error;
      }
    },
  };
};
