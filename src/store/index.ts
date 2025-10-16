import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import type { AppState, UserSettings, FocusSession, BreakSession, SessionStats } from '../types';

// Default settings based on requirements
const DEFAULT_SETTINGS: UserSettings = {
  focusDuration: 25, // minutes
  shortBreakDuration: 5, // minutes
  longBreakDuration: 15, // minutes
  cyclesPerLongBreak: 4,
  preAlertSeconds: 120, // 2 minutes
  strictMode: false,
  blockedApps: [],
  blockedWebsites: [],
};

export const useAppStore = create<AppState>()(
  devtools(
    (set, get) => ({
      // Initial state
      currentSession: null,
      currentBreak: null,
      isCommandPaletteOpen: false,
      isFocusWidgetVisible: false,
      isBreakOverlayVisible: false,
      isSettingsOpen: false,
      settings: DEFAULT_SETTINGS,
      stats: [],

      // Session actions
      setCurrentSession: (session: FocusSession | null) =>
        set(
          { currentSession: session },
          false,
          'setCurrentSession'
        ),

      setCurrentBreak: (breakSession: BreakSession | null) =>
        set(
          { currentBreak: breakSession },
          false,
          'setCurrentBreak'
        ),

      // UI actions
      toggleCommandPalette: () =>
        set(
          (state) => ({ isCommandPaletteOpen: !state.isCommandPaletteOpen }),
          false,
          'toggleCommandPalette'
        ),

      showFocusWidget: () =>
        set(
          { isFocusWidgetVisible: true },
          false,
          'showFocusWidget'
        ),

      hideFocusWidget: () =>
        set(
          { isFocusWidgetVisible: false },
          false,
          'hideFocusWidget'
        ),

      showBreakOverlay: () =>
        set(
          { isBreakOverlayVisible: true },
          false,
          'showBreakOverlay'
        ),

      hideBreakOverlay: () =>
        set(
          { isBreakOverlayVisible: false },
          false,
          'hideBreakOverlay'
        ),

      toggleSettings: () =>
        set(
          (state) => ({ isSettingsOpen: !state.isSettingsOpen }),
          false,
          'toggleSettings'
        ),

      // Settings actions
      updateSettings: (newSettings: Partial<UserSettings>) =>
        set(
          (state) => ({
            settings: { ...state.settings, ...newSettings },
          }),
          false,
          'updateSettings'
        ),

      // Stats actions
      setStats: (stats: SessionStats[]) =>
        set(
          { stats },
          false,
          'setStats'
        ),
    }),
    {
      name: 'pausa-store',
    }
  )
);

// Selectors for commonly used state combinations
export const useCurrentSession = () => useAppStore((state) => state.currentSession);
export const useCurrentBreak = () => useAppStore((state) => state.currentBreak);
export const useSettings = () => useAppStore((state) => state.settings);
export const useUIState = () => useAppStore((state) => ({
  isCommandPaletteOpen: state.isCommandPaletteOpen,
  isFocusWidgetVisible: state.isFocusWidgetVisible,
  isBreakOverlayVisible: state.isBreakOverlayVisible,
  isSettingsOpen: state.isSettingsOpen,
}));
export const useStats = () => useAppStore((state) => state.stats);
