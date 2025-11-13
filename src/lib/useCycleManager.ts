import { useAppStore } from "../store";
import { CycleManager } from "./cycleCommands";

/**
 * Hook to manage cycle actions
 * Note: Cycle synchronization and timer are handled globally by CycleSync component
 */
export const useCycleManager = () => {
  const { setCycleState } = useAppStore();

  return {
    // Starts the full routine (focus → break → focus ...) leveraging the orchestrator's auto-transitions
    startRoutine: async (options?: {
      resetCount?: boolean;
      overrideWorkHours?: boolean;
    }) => {
      try {
        if (options?.resetCount) {
          const stateAfterReset = await CycleManager.resetCycleCount();
          setCycleState(stateAfterReset);
        }
        const state = await CycleManager.startFocusSession(
          options?.overrideWorkHours
        );
        setCycleState(state);
      } catch (error) {
        console.error("Failed to start routine:", error);
        throw error;
      }
    },
    startFocusSession: async (overrideWorkHours?: boolean) => {
      try {
        const state = await CycleManager.startFocusSession(overrideWorkHours);
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
