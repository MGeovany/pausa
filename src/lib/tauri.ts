// Tauri API wrapper with error handling
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { TauriResult, AppEvent } from '../types';

// Generic invoke wrapper with error handling
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    const result = await invoke<TauriResult<T>>(command, args);

    if (!result.success && result.error) {
      throw new Error(`${result.error.type}: ${result.error.message}`);
    }

    return result.data as T;
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

// Tauri command definitions (will be implemented in backend)
export const tauriCommands = {
  // Session management
  startFocusSession: (strict: boolean) =>
    invokeCommand('start_focus_session', { strict }),

  pauseSession: () =>
    invokeCommand('pause_session'),

  resumeSession: () =>
    invokeCommand('resume_session'),

  endSession: () =>
    invokeCommand('end_session'),

  getCurrentSession: () =>
    invokeCommand('get_current_session'),

  // Settings management
  getSettings: () =>
    invokeCommand('get_settings'),

  updateSettings: (settings: Record<string, unknown>) =>
    invokeCommand('update_settings', { settings }),

  // Statistics
  getSessionStats: (days: number) =>
    invokeCommand('get_session_stats', { days }),

  // Window management
  showWindow: (windowType: string) =>
    invokeCommand('show_window', { windowType }),

  hideWindow: (windowType: string) =>
    invokeCommand('hide_window', { windowType }),

  // Emergency override
  verifyEmergencyPin: (pin: string) =>
    invokeCommand('verify_emergency_pin', { pin }),
} as const;
