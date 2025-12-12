import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { BreakSession, CycleEventData, CycleState } from "../../types";
import { useCycleState, useSettings } from "../../store";
import {
  activityCompletionTracker,
  breakActivityManager,
} from "../../lib/breakActivities";

interface UseBreakOverlayLogicParams {
  breakSession: BreakSession;
  cycleState?: CycleState | null;
  isStrictMode: boolean;
  emergencyKeyCombination?: string;
  onCompleteBreak: () => void;
  onEmergencyOverride: (pin: string) => Promise<boolean>;
}

export function useBreakOverlayLogic({
  breakSession,
  cycleState,
  isStrictMode,
  emergencyKeyCombination,
  onCompleteBreak,
  onEmergencyOverride,
}: UseBreakOverlayLogicParams) {
  const resolvedBreakType = useMemo<"short" | "long">(() => {
    const raw =
      (breakSession as any).type ||
      (breakSession as any).break_type ||
      (breakSession as any).breakType;
    return raw === "long" ? "long" : "short";
  }, [breakSession]);

  const [checklistCompleted, setChecklistCompleted] = useState<boolean[]>([]);
  const [showEmergencyModal, setShowEmergencyModal] = useState(false);
  const [bypassAttempts, setBypassAttempts] = useState(0);
  const storeCycleState = useCycleState();
  const settings = useSettings();
  const [remaining, setRemaining] = useState(breakSession.remaining);
  const [phase, setPhase] = useState<CycleState["phase"]>(
    resolvedBreakType === "short" ? "short_break" : "long_break"
  );
  const windowRef = useRef<ReturnType<typeof getCurrentWindow> | null>(null);
  const lastSyncRef = useRef<number>(Date.now());
  const breakCompletionHandledRef = useRef(false);
  const phaseRef = useRef(phase);

  const currentCycleState = cycleState || storeCycleState;

  const activity = useMemo(
    () =>
      breakSession.activity ||
      breakActivityManager.getActivityForBreak(
        resolvedBreakType,
        breakSession.duration
      ),
    [breakSession.activity, resolvedBreakType, breakSession.duration]
  );

  const logBypassAttempt = useCallback(
    async (method: string) => {
      const timestamp = new Date().toISOString();
      console.warn(
        `[BYPASS ATTEMPT] ${timestamp} - Method: ${method} - Session: ${breakSession.id}`
      );
      setBypassAttempts((prev) => prev + 1);

      try {
        await invoke("log_bypass_attempt", {
          sessionId: breakSession.id,
          method,
          timestamp,
        });
      } catch (error) {
        console.error("Failed to log bypass attempt:", error);
      }
    },
    [breakSession.id]
  );

  useEffect(() => {
    setRemaining(breakSession.remaining);
    setPhase(resolvedBreakType === "short" ? "short_break" : "long_break");
    phaseRef.current =
      resolvedBreakType === "short" ? "short_break" : "long_break";
    lastSyncRef.current = Date.now();
    breakCompletionHandledRef.current = false;
    setBypassAttempts(0);
    setChecklistCompleted([]);

    // Immediately reconcile with backend to avoid stale 0:00 on new breaks
    (async () => {
      try {
        const state = await invoke<CycleState>("get_cycle_state");
        if (state.phase === "short_break" || state.phase === "long_break") {
          setRemaining(state.remaining_seconds);
          setPhase(state.phase);
          phaseRef.current = state.phase;
          lastSyncRef.current = Date.now();
          breakCompletionHandledRef.current = false;
        }
      } catch (error) {
        console.error("❌ [BreakOverlay] Immediate sync failed:", error);
      }
    })();
  }, [breakSession.id, breakSession.remaining, resolvedBreakType]);

  useEffect(() => {
    let cancelled = false;

    const hydrateFromBackend = async () => {
      try {
        const liveBreak = await invoke<BreakSession | null>(
          "get_current_break"
        );
        if (cancelled || !liveBreak) return;

        const liveType =
          (liveBreak as any).type ||
          (liveBreak as any).break_type ||
          (liveBreak as any).breakType;
        const livePhase = liveType === "long" ? "long_break" : "short_break";

        setRemaining(liveBreak.remaining);
        setPhase(livePhase);
        phaseRef.current = livePhase;
        lastSyncRef.current = Date.now();
      } catch (error) {
        console.error(
          "❌ [BreakOverlay] Failed to hydrate break state:",
          error
        );
      }
    };

    if (breakSession.remaining <= 0) {
      hydrateFromBackend();
    }

    return () => {
      cancelled = true;
    };
  }, [breakSession.remaining]);

  useEffect(() => {
    let cancelled = false;

    const syncRemaining = async () => {
      if (cancelled) return;

      try {
        const state = await invoke<CycleState>("get_cycle_state");

        if (
          !cancelled &&
          (state.phase === "short_break" || state.phase === "long_break")
        ) {
          setRemaining(state.remaining_seconds);
          setPhase(state.phase);
          phaseRef.current = state.phase;
          lastSyncRef.current = Date.now();
          if (state.remaining_seconds > 0) {
            breakCompletionHandledRef.current = false;
          }

          if (state.remaining_seconds === 0) {
            cancelled = true;
            try {
              await invoke("hide_fullscreen_break_overlay");
            } catch (error) {
              console.error("❌ [BreakOverlay] Failed to hide:", error);
            }
          }
        } else if (
          !cancelled &&
          state.phase !== "short_break" &&
          state.phase !== "long_break"
        ) {
          cancelled = true;
          try {
            await invoke("hide_fullscreen_break_overlay");
          } catch (error) {
            console.error("❌ [BreakOverlay] Failed to hide:", error);
          }
        }
      } catch (error) {
        console.error("❌ [BreakOverlay] Sync error:", error);
      }
    };

    syncRemaining();
    const interval = setInterval(syncRemaining, 1000);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [breakSession.id]);

  useEffect(() => {
    lastSyncRef.current = Date.now();

    const fallback = setInterval(() => {
      const msSinceSync = Date.now() - lastSyncRef.current;
      if (msSinceSync > 1500) {
        setRemaining((prev) => (prev > 0 ? prev - 1 : 0));
      }
    }, 1000);

    return () => clearInterval(fallback);
  }, [breakSession.id]);

  useEffect(() => {
    const refocus = async (reason: string, shouldLog = false) => {
      try {
        const win = windowRef.current ?? getCurrentWindow();
        windowRef.current = win;
        await win.setAlwaysOnTop(true);
        await win.setFullscreen(true);
        await win.setFocus();
        if (shouldLog) {
          logBypassAttempt(`refocus_${reason}`);
        }
      } catch (error) {
        console.error("Failed to refocus window:", error);
      }
    };

    const handleBlur = (e: FocusEvent) => {
      e.preventDefault();
      logBypassAttempt("window_blur_detected");
      refocus("window_blur", true);
    };

    const handleVisibility = (e: Event) => {
      e.preventDefault?.();
      if (document.hidden) {
        logBypassAttempt("visibility_change_detected");
        refocus("visibility_change", true);
      }
    };

    window.addEventListener("blur", handleBlur, true);
    document.addEventListener("visibilitychange", handleVisibility, true);

    const interval = setInterval(() => refocus("interval"), 350);
    refocus("mount");

    return () => {
      window.removeEventListener("blur", handleBlur, true);
      document.removeEventListener("visibilitychange", handleVisibility, true);
      clearInterval(interval);
    };
  }, [isStrictMode, logBypassAttempt]);

  useEffect(() => {
    windowRef.current = getCurrentWindow();
  }, []);

  useEffect(() => {
    const blockKeyboardInput = (e: KeyboardEvent) => {
      if (emergencyKeyCombination) {
        const keys = emergencyKeyCombination
          .split("+")
          .map((k) => k.trim().toLowerCase());
        const mainKey = keys.find(
          (k) => !["cmd", "ctrl", "alt", "shift", "meta"].includes(k)
        );

        const needsCmd = keys.includes("cmd") || keys.includes("meta");
        const needsCtrl = keys.includes("ctrl");
        const needsAlt = keys.includes("alt");
        const needsShift = keys.includes("shift");

        const hasCmd = needsCmd && (e.metaKey || e.key === "Meta");
        const hasCtrl = needsCtrl && e.ctrlKey;
        const hasAlt = needsAlt && e.altKey;
        const hasShift = needsShift && e.shiftKey;

        const hasMainKey =
          !mainKey || e.key.toLowerCase() === mainKey.toLowerCase();

        const modifiersMatch =
          (needsCmd ? hasCmd : !e.metaKey) &&
          (needsCtrl ? hasCtrl : !e.ctrlKey) &&
          (needsAlt ? hasAlt : !e.altKey) &&
          (needsShift ? hasShift : !e.shiftKey) &&
          hasMainKey;

        if (modifiersMatch) {
          invoke("emergency_exit_strict_mode").catch((error) => {
            console.error("❌ [BreakOverlay] Emergency exit failed:", error);
          });

          return;
        }
      }

      e.preventDefault();
      e.stopPropagation();
      e.stopImmediatePropagation();

      if (e.metaKey || e.ctrlKey) {
        if (e.key === "q" || e.key === "Q") {
          logBypassAttempt("cmd_q_blocked");
        } else if (e.key === "w" || e.key === "W") {
          logBypassAttempt("cmd_w_blocked");
        } else if (e.key === "m" || e.key === "M") {
          logBypassAttempt("cmd_m_blocked");
        } else if (e.key === "Tab") {
          logBypassAttempt("cmd_tab_blocked");
        } else if (e.key === "h" || e.key === "H") {
          logBypassAttempt("cmd_h_blocked");
        } else if (e.key === "`" || e.key === "~") {
          logBypassAttempt("cmd_backtick_blocked");
        }
      }

      if (e.key.startsWith("F") && e.key.length <= 3) {
        logBypassAttempt(`function_key_blocked_${e.key}`);
      }

      if (e.key === "Escape" || e.key === "Esc") {
        logBypassAttempt("escape_blocked");
      }

      logBypassAttempt(`keyboard_blocked_${e.key}`);
    };

    window.addEventListener("keydown", blockKeyboardInput, true);
    window.addEventListener("keyup", blockKeyboardInput, true);
    window.addEventListener("keypress", blockKeyboardInput, true);

    return () => {
      window.removeEventListener("keydown", blockKeyboardInput, true);
      window.removeEventListener("keyup", blockKeyboardInput, true);
      window.removeEventListener("keypress", blockKeyboardInput, true);
    };
  }, [isStrictMode, emergencyKeyCombination, logBypassAttempt]);

  useEffect(() => {
    if (!isStrictMode) {
      return;
    }

    const blockMouseInput = (e: MouseEvent) => {
      if (e.type === "mousemove") {
        return;
      }

      e.preventDefault();
      e.stopPropagation();
      e.stopImmediatePropagation();

      logBypassAttempt(`mouse_blocked_${e.type}`);
    };

    window.addEventListener("click", blockMouseInput, true);
    window.addEventListener("mousedown", blockMouseInput, true);
    window.addEventListener("mouseup", blockMouseInput, true);
    window.addEventListener("dblclick", blockMouseInput, true);
    window.addEventListener("contextmenu", blockMouseInput, true);
    window.addEventListener("wheel", blockMouseInput, true);

    return () => {
      window.removeEventListener("click", blockMouseInput, true);
      window.removeEventListener("mousedown", blockMouseInput, true);
      window.removeEventListener("mouseup", blockMouseInput, true);
      window.removeEventListener("dblclick", blockMouseInput, true);
      window.removeEventListener("contextmenu", blockMouseInput, true);
      window.removeEventListener("wheel", blockMouseInput, true);
    };
  }, [isStrictMode, logBypassAttempt]);

  useEffect(() => {
    if (!isStrictMode) return;

    const preventWindowClose = (e: BeforeUnloadEvent) => {
      e.preventDefault();
      e.returnValue = "";
      logBypassAttempt("window_close_blocked");
    };

    const preventVisibilityChange = () => {
      if (document.hidden) {
        logBypassAttempt("visibility_change_blocked");
      }
    };

    window.addEventListener("beforeunload", preventWindowClose);
    document.addEventListener("visibilitychange", preventVisibilityChange);

    return () => {
      window.removeEventListener("beforeunload", preventWindowClose);
      document.removeEventListener("visibilitychange", preventVisibilityChange);
    };
  }, [isStrictMode, logBypassAttempt]);

  useEffect(() => {
    const isBreakPhase = phase === "short_break" || phase === "long_break";
    if (
      !isStrictMode ||
      remaining > 0 ||
      breakCompletionHandledRef.current ||
      isBreakPhase
    ) {
      return;
    }

    const handleBreakEnd = async () => {
      breakCompletionHandledRef.current = true;
      try {
        await invoke("hide_fullscreen_break_overlay");
        onCompleteBreak();
      } catch (error) {
        console.error("❌ [BreakOverlay] Failed to unlock system:", error);
        onCompleteBreak();
      }
    };

    handleBreakEnd();
  }, [isStrictMode, remaining, onCompleteBreak, phase]);

  useEffect(() => {
    const isBreakPhase = phase === "short_break" || phase === "long_break";
    if (
      isStrictMode ||
      remaining > 0 ||
      breakCompletionHandledRef.current ||
      isBreakPhase
    ) {
      return;
    }

    const finalizeBreak = async () => {
      breakCompletionHandledRef.current = true;
      try {
        await invoke("hide_fullscreen_break_overlay");
      } catch (error) {
        console.error("❌ [BreakOverlay] Failed to hide overlay:", error);
      } finally {
        onCompleteBreak();
      }
    };

    finalizeBreak();
  }, [isStrictMode, remaining, onCompleteBreak, phase]);

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen<CycleEventData>("cycle-event", (event) => {
          const data = event.payload;

          if (data.type === "phase_started") {
            setPhase(data.phase);
            phaseRef.current = data.phase;
            if (data.phase === "short_break" || data.phase === "long_break") {
              setRemaining(data.duration);
              lastSyncRef.current = Date.now();
              breakCompletionHandledRef.current = false;
            } else {
              setRemaining(data.duration);
            }
            return;
          }

          if (data.type === "tick") {
            lastSyncRef.current = Date.now();
            const currentPhase = phaseRef.current;
            if (
              currentPhase === "short_break" ||
              currentPhase === "long_break"
            ) {
              setRemaining(data.remaining);
            }
            return;
          }

          if (data.type === "phase_ended") {
            const nextPhase: CycleState["phase"] =
              data.phase === "short_break" || data.phase === "long_break"
                ? "focus"
                : "idle";
            setPhase(nextPhase);
            phaseRef.current = nextPhase;
          }
        });
      } catch (error) {
        console.error(
          "❌ [BreakOverlay] Failed to set up cycle-event listener:",
          error
        );
      }
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  const handleEmergencyOverride = useCallback(
    async (pin: string): Promise<boolean> => {
      const success = await onEmergencyOverride(pin);
      if (success) {
        setShowEmergencyModal(false);
        logBypassAttempt("emergency_override_success");
      } else {
        logBypassAttempt("emergency_override_failed");
      }
      return success;
    },
    [onEmergencyOverride, logBypassAttempt]
  );

  const handleChecklistUpdate = useCallback(
    (completedItems: boolean[]) => {
      setChecklistCompleted(completedItems);

      activityCompletionTracker.recordCompletion(
        activity.title,
        completedItems,
        breakSession.id
      );
    },
    [activity.title, breakSession.id]
  );

  const canCompleteBreak = useMemo(
    () =>
      Math.max(remaining, 0) <= 0 ||
      (checklistCompleted.length > 0 &&
        checklistCompleted.every((item) => item)),
    [remaining, checklistCompleted]
  );

  const showCompletionInterface = useMemo(
    () => Math.max(remaining, 0) <= 0,
    [remaining]
  );

  const accentColor = useMemo(
    () => (resolvedBreakType === "long" ? "amber" : "blue"),
    [resolvedBreakType]
  );

  const normalModeBg = useMemo(
    () =>
      resolvedBreakType === "long"
        ? "bg-gradient-to-br from-amber-900/40 via-gray-900 to-gray-900"
        : "bg-gray-900",
    [resolvedBreakType]
  );

  const clampedRemaining = Math.max(remaining, 0);

  return {
    resolvedBreakType,
    remaining,
    clampedRemaining,
    phase,
    activity,
    currentCycleState,
    settings,
    checklistCompleted,
    canCompleteBreak,
    showCompletionInterface,
    accentColor,
    normalModeBg,
    bypassAttempts,
    showEmergencyModal,
    setShowEmergencyModal,
    handleChecklistUpdate,
    handleEmergencyOverride,
  };
}
