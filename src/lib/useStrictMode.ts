import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { StrictModeState } from '../types';
import { useAppStore } from '../store';

/**
 * Hook for managing strict mode functionality
 * Provides methods to activate/deactivate strict mode and manage its state
 */
export const useStrictMode = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Get state and setter from global store
  const state = useAppStore((state) => state.strictModeState);
  const setStrictModeState = useAppStore((state) => state.setStrictModeState);

  /**
   * Refresh the strict mode state from the backend
   */
  const refreshState = useCallback(async () => {
    try {
      setError(null);
      const newState = await invoke<StrictModeState>('get_strict_mode_state');
      setStrictModeState(newState);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error('[useStrictMode] Failed to refresh state:', errorMessage);
      setError(errorMessage);
    }
  }, [setStrictModeState]);

  /**
   * Activate strict mode
   */
  const activateStrictMode = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      await invoke('activate_strict_mode');
      await refreshState();
      console.log('✅ [useStrictMode] Strict mode activated');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error('[useStrictMode] Failed to activate:', errorMessage);
      setError(errorMessage);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, [refreshState]);

  /**
   * Deactivate strict mode
   */
  const deactivateStrictMode = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      await invoke('deactivate_strict_mode');
      await refreshState();
      console.log('✅ [useStrictMode] Strict mode deactivated');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error('[useStrictMode] Failed to deactivate:', errorMessage);
      setError(errorMessage);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, [refreshState]);

  /**
   * Show the menu bar popover
   */
  const showMenuBarPopover = useCallback(async () => {
    try {
      setError(null);
      await invoke('show_menu_bar_popover');
      await refreshState();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error('[useStrictMode] Failed to show popover:', errorMessage);
      setError(errorMessage);
      throw err;
    }
  }, [refreshState]);

  /**
   * Hide the menu bar popover
   */
  const hideMenuBarPopover = useCallback(async () => {
    try {
      setError(null);
      await invoke('hide_menu_bar_popover');
      await refreshState();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error('[useStrictMode] Failed to hide popover:', errorMessage);
      setError(errorMessage);
      throw err;
    }
  }, [refreshState]);

  /**
   * Stop the break transition countdown
   */
  const stopBreakTransitionCountdown = useCallback(async () => {
    try {
      setError(null);
      await invoke('stop_break_transition_countdown');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error('[useStrictMode] Failed to stop countdown:', errorMessage);
      setError(errorMessage);
      throw err;
    }
  }, []);

  /**
   * Start break from transition window
   */
  const startBreakFromTransition = useCallback(async () => {
    try {
      setError(null);
      await invoke('start_break_from_transition');
      await refreshState();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error('[useStrictMode] Failed to start break:', errorMessage);
      setError(errorMessage);
      throw err;
    }
  }, [refreshState]);

  /**
   * Emergency exit from strict mode
   */
  const emergencyExit = useCallback(async () => {
    try {
      setError(null);
      await invoke('emergency_exit_strict_mode');
      await refreshState();
      console.log('🚨 [useStrictMode] Emergency exit completed');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error('[useStrictMode] Failed to emergency exit:', errorMessage);
      setError(errorMessage);
      throw err;
    }
  }, [refreshState]);

  // Load initial state on mount
  useEffect(() => {
    refreshState();
  }, [refreshState]);

  return {
    state,
    isLoading,
    error,
    activateStrictMode,
    deactivateStrictMode,
    showMenuBarPopover,
    hideMenuBarPopover,
    stopBreakTransitionCountdown,
    startBreakFromTransition,
    emergencyExit,
    refreshState,
  };
};
