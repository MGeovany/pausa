// Tauri API wrapper with error handling
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { AppEvent, FocusSession, BreakSession, UserSettings, SessionStats } from '../types';

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

// Event listener setup
export function setupEventListeners(
  onEvent: (event: AppEvent) => void
): Promise<() => void> {
  return listen<AppEvent>('app-event', (event) => {
    onEvent(event.payload);
  });
}

// Tauri command definitions
export const tauriCommands = {
  // Session management
  startFocusSession: (strict: boolean) =>
    invokeCommand<FocusSession>('start_focus_session', { strict }),

  pauseSession: () =>
    invokeCommand<void>('pause_session'),

  resumeSession: () =>
    invokeCommand<void>('resume_session'),

  endSession: () =>
    invokeCommand<void>('end_session'),

  getCurrentSession: () =>
    invokeCommand<FocusSession | null>('get_current_session'),

  getCurrentBreak: () =>
    invokeCommand<BreakSession | null>('get_current_break'),

  completeBreak: () =>
    invokeCommand<void>('complete_break'),

  // Settings management
  getSettings: () =>
    invokeCommand<UserSettings>('get_settings'),

  updateSettings: (settings: UserSettings) =>
    invokeCommand<void>('update_settings', { settings }),

  // Statistics
  getSessionStats: (days: number) =>
    invokeCommand<SessionStats[]>('get_session_stats', { days }),

  // State information
  getAppState: () =>
    invokeCommand<string>('get_app_state'),

  // Testing
  testStateManager: () =>
    invokeCommand<string>('test_state_manager'),

  getDatabaseStats: () =>
    invokeCommand<string>('get_database_stats'),
} as const;
