// Tauri API wrapper with error handling
import { invoke } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";
import type {
  AppEvent,
  FocusSession,
  BreakSession,
  UserSettings,
  SessionStats,
  BreakActivity,
} from "../types";

// Generic invoke wrapper with error handling
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    // Tauri automatically handles Result<T, String> from Rust
    const result = await invoke<T>(command, args);
    return result;
  } catch (error) {
    console.error(`Failed to invoke command ${command}:`, error);
    throw error;
  }
}

// Event listener setup with multiple event types
export function setupEventListeners(
  onEvent: (event: AppEvent) => void
): Promise<() => void> {
  const listeners: Promise<() => void>[] = [];

  // Listen for session updates
  listeners.push(
    listen<FocusSession>("session-update", (event) => {
      onEvent({
        type: "session-update",
        session: event.payload,
      });
    })
  );

  // Listen for break updates
  listeners.push(
    listen<BreakSession>("break-update", (event) => {
      onEvent({
        type: "break-update",
        breakSession: event.payload,
      });
    })
  );

  // Listen for state changes
  listeners.push(
    listen<{ from: string; to: string }>("state-change", (event) => {
      onEvent({
        type: "state-change",
        from: event.payload.from as any,
        to: event.payload.to as any,
      });
    })
  );

  // Return a function that unsubscribes from all listeners
  return Promise.all(listeners).then((unsubscribeFunctions) => {
    return () => {
      unsubscribeFunctions.forEach((unsubscribe) => unsubscribe());
    };
  });
}

// Emit events to backend (if needed)
export async function emitEvent(
  eventName: string,
  payload?: any
): Promise<void> {
  try {
    await emit(eventName, payload);
  } catch (error) {
    console.error(`Failed to emit event ${eventName}:`, error);
    throw error;
  }
}

// Tauri command definitions
export const tauriCommands = {
  // Session management
  startFocusSession: (strict: boolean) =>
    invokeCommand<FocusSession>("start_focus_session", { strict }),

  pauseSession: () => invokeCommand<void>("pause_session"),

  resumeSession: () => invokeCommand<void>("resume_session"),

  endSession: () => invokeCommand<void>("end_session"),

  getCurrentSession: () =>
    invokeCommand<FocusSession | null>("get_current_session"),

  getCurrentBreak: () =>
    invokeCommand<BreakSession | null>("get_current_break"),

  completeBreak: () => invokeCommand<void>("complete_break"),

  // Emergency override
  verifyEmergencyPin: (pin: string) =>
    invokeCommand<boolean>("verify_emergency_pin", { pin }),

  // Break activities
  getBreakActivity: (breakType: "short" | "long", duration: number) =>
    invokeCommand<BreakActivity>("get_break_activity", { breakType, duration }),

  getCustomActivities: () =>
    invokeCommand<BreakActivity[]>("get_custom_activities"),

  addCustomActivity: (activity: BreakActivity) =>
    invokeCommand<void>("add_custom_activity", { activity }),

  updateActivity: (oldTitle: string, newActivity: BreakActivity) =>
    invokeCommand<void>("update_activity", { oldTitle, newActivity }),

  removeActivity: (title: string) =>
    invokeCommand<boolean>("remove_activity", { title }),

  // Settings management
  getSettings: () => invokeCommand<UserSettings>("get_settings"),

  updateSettings: (settings: UserSettings) =>
    invokeCommand<void>("update_settings", { settings }),

  // Statistics
  getSessionStats: (days: number) =>
    invokeCommand<SessionStats[]>("get_session_stats", { days }),

  // State information
  getAppState: () => invokeCommand<string>("get_app_state"),

  // Testing
  testStateManager: () => invokeCommand<string>("test_state_manager"),

  getDatabaseStats: () => invokeCommand<string>("get_database_stats"),

  // Window management
  isWindowVisible: (windowType: string) =>
    invokeCommand<boolean>("is_window_visible", { windowType }),

  // Onboarding commands
  startOnboarding: () => invokeCommand<string>("start_onboarding"),

  nextOnboardingStep: (stepData?: any) =>
    invokeCommand<string>("next_onboarding_step", { stepData }),

  previousOnboardingStep: () =>
    invokeCommand<string>("previous_onboarding_step"),

  completeOnboarding: (finalConfig: any) =>
    invokeCommand<void>("complete_onboarding", { finalConfig }),

  getOnboardingStatus: () => invokeCommand<boolean>("get_onboarding_status"),

  // Strict mode commands
  registerEmergencyHotkey: (combination: string) =>
    invokeCommand<void>("register_emergency_hotkey", { combination }),

  unregisterEmergencyHotkey: () =>
    invokeCommand<void>("unregister_emergency_hotkey"),

  emergencyExitStrictMode: () =>
    invokeCommand<void>("emergency_exit_strict_mode"),

  getStrictModeConfig: () =>
    invokeCommand<{
      strict_mode: boolean;
      emergency_key_combination: string | null;
    }>("get_strict_mode_config"),

  saveStrictModeConfig: (config: {
    strict_mode: boolean;
    emergency_key_combination?: string | null;
  }) => invokeCommand<void>("save_strict_mode_config", { config }),
} as const;
