import { invoke } from "@tauri-apps/api/core";
import type { CycleState } from "../types";

/**
 * Cycle management commands for interacting with the Rust backend
 */
export class CycleManager {
  /**
   * Initialize the cycle orchestrator with current user settings
   */
  static async initialize(): Promise<CycleState> {
    try {
      const state = await invoke<CycleState>("initialize_cycle_orchestrator");
      console.log("✅ Cycle orchestrator initialized:", state);
      return state;
    } catch (error) {
      console.error("❌ Failed to initialize cycle orchestrator:", error);
      throw error;
    }
  }

  /**
   * Start a focus session with optional work hours override
   */
  static async startFocusSession(
    overrideWorkHours?: boolean
  ): Promise<CycleState> {
    try {
      const state = await invoke<CycleState>("start_focus_session", {
        overrideWorkHours: overrideWorkHours || false,
      });
      console.log("✅ Focus session started:", state);
      return state;
    } catch (error) {
      console.error("❌ Failed to start focus session:", error);
      throw error;
    }
  }

  /**
   * Start a break session (short or long)
   */
  static async startBreakSession(forceLong?: boolean): Promise<CycleState> {
    try {
      const state = await invoke<CycleState>("start_break_session", {
        forceLong: forceLong || false,
      });
      console.log("✅ Break session started:", state);
      return state;
    } catch (error) {
      console.error("❌ Failed to start break session:", error);
      throw error;
    }
  }

  /**
   * Pause the current cycle
   */
  static async pause(): Promise<CycleState> {
    try {
      const state = await invoke<CycleState>("pause_cycle");
      console.log("✅ Cycle paused:", state);
      return state;
    } catch (error) {
      console.error("❌ Failed to pause cycle:", error);
      throw error;
    }
  }

  /**
   * Resume the current cycle
   */
  static async resume(): Promise<CycleState> {
    try {
      const state = await invoke<CycleState>("resume_cycle");
      console.log("✅ Cycle resumed:", state);
      return state;
    } catch (error) {
      console.error("❌ Failed to resume cycle:", error);
      throw error;
    }
  }

  /**
   * End the current session
   */
  static async endSession(completed: boolean): Promise<CycleState> {
    try {
      const state = await invoke<CycleState>("end_cycle_session", {
        completed,
      });
      console.log("✅ Cycle session ended:", state);
      return state;
    } catch (error) {
      console.error("❌ Failed to end cycle session:", error);
      throw error;
    }
  }

  /**
   * Get the current cycle state
   */
  static async getState(): Promise<CycleState> {
    try {
      const state = await invoke<CycleState>("get_cycle_state");
      return state;
    } catch (error) {
      console.error("❌ Failed to get cycle state:", error);
      throw error;
    }
  }

  /**
   * Trigger a timer tick (should be called every second)
   */
  static async tick(): Promise<CycleState> {
    try {
      const state = await invoke<CycleState>("cycle_tick");
      return state;
    } catch (error) {
      console.error("❌ Failed to tick cycle:", error);
      throw error;
    }
  }

  /**
   * Reset the cycle counter
   */
  static async resetCycleCount(): Promise<CycleState> {
    try {
      const state = await invoke<CycleState>("reset_cycle_count");
      console.log("✅ Cycle count reset:", state);
      return state;
    } catch (error) {
      console.error("❌ Failed to reset cycle count:", error);
      throw error;
    }
  }

  /**
   * Get work schedule information
   */
  static async getWorkScheduleInfo(): Promise<WorkScheduleInfo | null> {
    try {
      const info = await invoke<WorkScheduleInfo | null>(
        "get_work_schedule_info"
      );
      return info;
    } catch (error) {
      console.error("❌ Failed to get work schedule info:", error);
      throw error;
    }
  }
}

/**
 * Work schedule information
 */
export interface WorkScheduleInfo {
  start_time: string | null;
  end_time: string | null;
  timezone: string;
  is_within_hours: boolean;
}

/**
 * Work hours compliance statistics
 */
export interface WorkHoursStats {
  total_sessions: number;
  within_work_hours: number;
  outside_work_hours: number;
  compliance_percentage: number;
  total_focus_minutes_within: number;
  total_focus_minutes_outside: number;
  period_start: string;
  period_end: string;
}

/**
 * Get work hours compliance statistics
 */
export async function getWorkHoursStats(
  days?: number
): Promise<WorkHoursStats> {
  try {
    const stats = await invoke<WorkHoursStats>("get_work_hours_stats", {
      days: days || 30,
    });
    return stats;
  } catch (error) {
    console.error("❌ Failed to get work hours stats:", error);
    throw error;
  }
}
