import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import {
  LogOut,
  Home,
  BarChart3,
  Settings,
  Play,
  Coffee,
  Pause,
  RefreshCcw,
  Square,
  Target,
  TrendingUp,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useCycleManager } from "../lib/useCycleManager";
import { useCycleState, useAppStore, useStrictModeState } from "../store";
import { tauriCommands } from "../lib/tauri";
import { CycleManager } from "../lib/cycleCommands";
import { useStrictMode } from "../lib/useStrictMode";
import type { SessionStats } from "../types";

interface UserInfo {
  name: string;
  email: string;
  picture: string;
}

export default function Dashboard() {
  const navigate = useNavigate();
  const [userInfo, setUserInfo] = useState<UserInfo | null>(null);
  const [avatarError, setAvatarError] = useState(false);
  const [todayStats, setTodayStats] = useState<SessionStats | null>(null);
  const cycleState = useCycleState();
  const strictModeState = useStrictModeState();
  const { setCycleState, updateSettings, settings } = useAppStore();
  const {
    startRoutine,
    pauseCycle,
    resumeCycle,
    endSession,
    resetCycleCount,
  } = useCycleManager();
  const { activateStrictMode } = useStrictMode();

  // Load settings from database on mount
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const settings = await tauriCommands.getSettings();
        // Tauri automatically converts snake_case to camelCase
        updateSettings({
          focusDuration: settings.focusDuration,
          shortBreakDuration: settings.shortBreakDuration,
          longBreakDuration: settings.longBreakDuration,
          cyclesPerLongBreak: settings.cyclesPerLongBreak,
          preAlertSeconds: settings.preAlertSeconds,
          strictMode: settings.strictMode,
          pinHash: settings.pinHash,
          emergencyKeyCombination: settings.emergencyKeyCombination,
          blockedApps: settings.blockedApps || [],
          blockedWebsites: settings.blockedWebsites || [],
        });
      } catch (error) {
        console.error("Failed to load settings:", error);
        // Continue with default settings if loading fails
      }
    };

    loadSettings();
  }, [updateSettings]);

  // Sync cycle state when Dashboard mounts to ensure we have the latest state
  useEffect(() => {
    const syncCycleState = async () => {
      try {
        const state = await CycleManager.getState();
        setCycleState(state);
      } catch (error) {
        console.error("Failed to sync cycle state:", error);
      }
    };

    syncCycleState();
  }, [setCycleState]);

  useEffect(() => {
    const fetchUserInfo = async () => {
      try {
        const info = await invoke<UserInfo>("get_user_info");
        if (info) {
          setUserInfo(info);
        }
      } catch (error) {
        console.error("Error fetching user info:", error);
      }
    };

    fetchUserInfo();
  }, []);

  useEffect(() => {
    setAvatarError(false);
  }, [userInfo?.picture]);

  useEffect(() => {
    const loadTodayStats = async () => {
      try {
        console.log("📊 [Dashboard] Loading today's stats...");
        const stats = await tauriCommands.getSessionStats(1);
        console.log("📊 [Dashboard] All stats:", stats);
        const today = new Date().toISOString().split("T")[0];
        console.log("📊 [Dashboard] Today's date:", today);
        const todayStat = stats.find((stat) => stat.date === today);
        console.log("📊 [Dashboard] Today's stat:", todayStat);
        setTodayStats(todayStat || null);
      } catch (error) {
        console.error("❌ [Dashboard] Error loading today stats:", error);
      }
    };

    loadTodayStats();

    // Listen for refresh-stats events
    const handleRefreshStats = () => {
      console.log("🔄 [Dashboard] Refresh stats event received");
      loadTodayStats();
    };
    window.addEventListener("refresh-stats", handleRefreshStats);

    // Refresh every 5 seconds to keep stats updated
    const interval = setInterval(loadTodayStats, 5000);

    return () => {
      clearInterval(interval);
      window.removeEventListener("refresh-stats", handleRefreshStats);
    };
  }, [cycleState?.cycle_count, cycleState?.phase]);

  const handleLogout = async () => {
    try {
      await invoke("logout");
      navigate("/");
    } catch (error) {
      console.error("Error logging out:", error);
    }
  };

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, "0")}:${secs
      .toString()
      .padStart(2, "0")}`;
  };

  const phaseLabel = useMemo(() => {
    if (!cycleState) return "Idle";
    switch (cycleState.phase) {
      case "focus":
        return "Focus";
      case "short_break":
        return "Short Break";
      case "long_break":
        return "Long Break";
      default:
        return "Idle";
    }
  }, [cycleState]);

  const userInitial = useMemo(() => {
    if (!userInfo?.name) return "?";
    return userInfo.name.trim().charAt(0).toUpperCase();
  }, [userInfo?.name]);

  // Handle starting routine with strict mode support
  const handleStartRoutine = async () => {
    try {
      // If strict mode is enabled in settings, activate the orchestrator
      if (settings.strictMode) {
        console.log("🔒 [Dashboard] Strict mode enabled, activating orchestrator");
        await activateStrictMode();
      }
      
      // Start the focus session
      await startRoutine();
      
      console.log("✅ [Dashboard] Routine started", { strictMode: settings.strictMode });
    } catch (error) {
      console.error("❌ [Dashboard] Failed to start routine:", error);
    }
  };

  return (
    <div className="min-h-screen bg-gray-950 text-gray-200">
      {/* App Shell */}
      <div className="flex">
        {/* Sidebar */}
        <aside className="hidden md:flex h-screen w-16 flex-col items-center justify-between py-4 border-r border-gray-800 bg-gray-900/40 sticky top-0">
          <div className="flex flex-col items-center gap-4">
            <div className="w-8 h-8 rounded-lg bg-gray-800 border border-gray-700 flex items-center justify-center text-md font-black text-gray-300">
              P
            </div>
            <nav className="flex flex-col items-center gap-2">
              <button
                onClick={() => navigate("/dashboard")}
                className="p-2 rounded-lg bg-gray-800"
                title="Home"
              >
                <Home className="w-5 h-5" />
              </button>
              <button
                onClick={() => navigate("/stats")}
                className="p-2 rounded-lg hover:bg-gray-800"
                title="Stats"
              >
                <BarChart3 className="w-5 h-5" />
              </button>
              <button
                onClick={() => navigate("/settings")}
                className="p-2 rounded-lg hover:bg-gray-800"
                title="Settings"
              >
                <Settings className="w-5 h-5" />
              </button>
            </nav>
          </div>
          <button
            onClick={handleLogout}
            className="p-2 rounded-lg hover:bg-gray-800"
            title="Logout"
          >
            <LogOut className="w-5 h-5" />
          </button>
        </aside>

        {/* Main */}
        <main className="flex-1">
          {/* Header */}
          <header className="sticky top-0 z-10 bg-gray-950/90 border-b border-gray-900">
            <div className="px-6 md:px-8 py-4 flex items-center justify-between">
              <div className="text-sm text-gray-400">
                Welcome{userInfo?.name ? "," : ""}{" "}
                <span className="text-gray-100 font-medium">
                  {userInfo?.name ?? "Pausa"}
                </span>
              </div>
              {userInfo &&
                (avatarError || !userInfo.picture ? (
                  <div className="w-8 h-8 rounded-full border border-gray-800 bg-gray-900 flex items-center justify-center text-sm font-semibold text-gray-400">
                    {userInitial}
                  </div>
                ) : (
                  <img
                    src={userInfo.picture}
                    alt={userInfo.name}
                    className="w-8 h-8 rounded-full border border-gray-800 object-cover"
                    onError={() => setAvatarError(true)}
                  />
                ))}
            </div>
          </header>

          {/* Content */}
          <div className="mx-auto w-full max-w-5xl px-6 py-4">
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
              {/* Quick Actions / Timer */}
              <section className="lg:col-span-2 bg-gray-900/40 border border-gray-800 rounded-xl p-6 md:p-8">
                <div className="flex items-center justify-between mb-4">
                  <h2 className="text-sm font-semibold text-gray-300 flex items-center gap-2">
                    Session
                    {strictModeState?.isActive && (
                      <span className="inline-flex items-center gap-1.5 text-xs text-amber-300 bg-amber-500/20 border border-amber-500/30 rounded-full px-3 py-1 font-medium animate-pulse-glow">
                        <span className="w-1.5 h-1.5 bg-amber-400 rounded-full animate-pulse"></span>
                        🔒 Strict Mode
                      </span>
                    )}
                  </h2>
                  <div className="text-xs text-gray-500">
                    {cycleState ? phaseLabel : "Loading…"}
                  </div>
                </div>

                <div className="flex flex-col items-center justify-center gap-4 py-6">
                  <div className="text-center">
                    <div className="text-5xl sm:text-6xl md:text-7xl font-mono font-semibold">
                      {cycleState
                        ? cycleState.phase === "idle"
                          ? "--:--"
                          : formatTime(cycleState.remaining_seconds)
                        : "--:--"}
                    </div>
                    <div className="text-xs text-gray-500 mt-2">
                      {cycleState
                        ? cycleState.is_running
                          ? "Running"
                          : cycleState.phase === "idle"
                          ? "Idle"
                          : "Paused"
                        : "Syncing…"}
                    </div>
                  </div>

                  <div className="flex items-center justify-center gap-2 flex-wrap">
                    {cycleState?.phase === "idle" && (
                      <>
                        <button
                          onClick={handleStartRoutine}
                          disabled={!cycleState?.can_start}
                          className={`inline-flex items-center gap-2 text-gray-100 border rounded-lg px-4 py-2 text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-all ${
                            settings.strictMode
                              ? 'bg-amber-600 hover:bg-amber-700 border-amber-500 shadow-lg shadow-amber-500/20'
                              : 'bg-gray-800 hover:bg-gray-700 border-gray-800'
                          }`}
                        >
                          <Play className="w-4 h-4" />
                          Start Focus
                          {settings.strictMode && (
                            <span className="ml-1 inline-flex items-center gap-1 text-xs text-amber-200 bg-amber-700/30 rounded px-1.5 py-0.5">
                              🔒
                            </span>
                          )}
                        </button>
                        {settings.strictMode && (
                          <div className="flex items-center justify-center gap-1.5 text-xs text-amber-300 w-full text-center mt-2 animate-pulse">
                            <span className="w-1.5 h-1.5 bg-amber-400 rounded-full"></span>
                            Strict mode will minimize to menu bar
                          </div>
                        )}
                      </>
                    )}

                    {cycleState && cycleState.phase !== "idle" && (
                      <>
                        {!cycleState.is_running ? (
                          <button
                            onClick={() => resumeCycle()}
                            className="inline-flex items-center gap-2 bg-gray-800 hover:bg-gray-700 text-gray-100 border border-gray-800 rounded-lg px-4 py-2 text-sm font-medium"
                          >
                            <Play className="w-4 h-4" />
                            Resume
                          </button>
                        ) : (
                          <button
                            onClick={() => pauseCycle()}
                            className="inline-flex items-center gap-2 bg-gray-900 hover:bg-gray-800 border border-gray-800 rounded-lg px-4 py-2 text-sm"
                          >
                            <Pause className="w-4 h-4" />
                            Pause
                          </button>
                        )}
                        <button
                          onClick={() => endSession(false)}
                          className="inline-flex items-center gap-2 bg-gray-900 hover:bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-sm text-red-300 hover:text-red-200"
                        >
                          <Square className="w-4 h-4" />
                          End
                        </button>
                      </>
                    )}
                  </div>
                </div>
              </section>

              {/* Today's Progress */}
              <section className="bg-gray-900/40 border border-gray-800 rounded-xl p-5">
                <h2 className="text-sm font-semibold text-gray-300 mb-4 flex items-center gap-2">
                  <Target className="w-4 h-4" />
                  Today's Progress
                </h2>
                <div className="grid grid-cols-2 gap-3 auto-rows-fr">
                  <div className="rounded-lg border border-gray-800 bg-gray-900/60 p-4 sm:p-5 md:p-6 col-span-2">
                    <div className="flex items-center justify-between mb-4">
                      <div className="flex items-center gap-2">
                        <TrendingUp className="w-4 h-4 text-amber-400" />
                        <div className="text-[10px] uppercase tracking-wide text-gray-500">
                          Cycle Progress
                        </div>
                      </div>
                      <button
                        onClick={() => resetCycleCount()}
                        className="inline-flex items-center gap-1.5 rounded-lg border border-gray-800 bg-gray-900/60 px-2.5 py-1.5 text-[11px] text-gray-300 hover:text-white hover:bg-gray-900 transition-colors"
                        title="Reset day"
                      >
                        <RefreshCcw className="w-3.5 h-3.5" />
                        Reset day
                      </button>
                    </div>
                    {(() => {
                      const cyclesPerLongBreak =
                        settings.cyclesPerLongBreak || 4;
                      const cyclesCompleted = cycleState?.cycle_count || 0;

                      // Current cycle in the group (1-based) accounting for current phase
                      // If we're in focus, we are working on the next focus of the group
                      const focusSessionsInGroup =
                        cyclesCompleted +
                        (cycleState?.phase === "focus" ? 1 : 0);
                      const cycleInGroup = Math.max(
                        1,
                        ((focusSessionsInGroup - 1) % cyclesPerLongBreak) + 1
                      );

                      // Generate steps: Focus -> Break -> Focus -> Break -> ... -> Long Break
                      const steps: Array<{
                        type: "focus" | "break" | "long_break";
                        index: number;
                        completed: boolean;
                      }> = [];
                      for (let i = 0; i < cyclesPerLongBreak; i++) {
                        steps.push({
                          type: "focus",
                          index: i + 1,
                          completed: cyclesCompleted > i,
                        });
                        if (i < cyclesPerLongBreak - 1) {
                          steps.push({
                            type: "break",
                            index: i + 1,
                            completed: cyclesCompleted > i,
                          });
                        } else {
                          steps.push({
                            type: "long_break",
                            index: i + 1,
                            completed: cyclesCompleted > i,
                          });
                        }
                      }

                      // Current step index (0-based) based on current phase
                      let currentStepIndex = 0;
                      if (cycleState?.phase === "focus") {
                        currentStepIndex = Math.min(
                          cyclesCompleted * 2,
                          steps.length - 1
                        );
                      } else if (cycleState?.phase === "short_break") {
                        currentStepIndex = Math.min(
                          Math.max(0, cyclesCompleted * 2 - 1),
                          steps.length - 1
                        );
                      } else if (cycleState?.phase === "long_break") {
                        currentStepIndex = steps.length - 1;
                      } else {
                        // idle: if we've completed a full group (long break done), show "Cycle Complete!"
                        if (
                          cyclesCompleted > 0 &&
                          cyclesCompleted % cyclesPerLongBreak === 0
                        ) {
                          currentStepIndex = steps.length; // sentinel to trigger "Cycle Complete!"
                        } else {
                          // otherwise show next focus position
                          currentStepIndex = Math.min(
                            cyclesCompleted * 2,
                            steps.length - 1
                          );
                        }
                      }
                      const isReadyForLongBreak =
                        cycleInGroup === cyclesPerLongBreak;

                      return (
                        <>
                          {/* Status row: Now / Next / Step */}
                          {(() => {
                            const nowPhase = cycleState?.phase || "idle";
                            const nowType =
                              nowPhase === "focus"
                                ? "focus"
                                : nowPhase === "short_break"
                                ? "break"
                                : nowPhase === "long_break"
                                ? "long_break"
                                : null;
                            let nextType:
                              | "focus"
                              | "break"
                              | "long_break"
                              | null = null;
                            if (nowPhase === "focus") {
                              nextType =
                                (cyclesCompleted + 1) % cyclesPerLongBreak === 0
                                  ? "long_break"
                                  : "break";
                            } else if (nowPhase === "short_break") {
                              nextType = "focus";
                            } else if (nowPhase === "idle") {
                              nextType =
                                cyclesCompleted > 0 &&
                                cyclesCompleted % cyclesPerLongBreak === 0
                                  ? null
                                  : "focus";
                            }

                            const labelFor = (
                              t: "focus" | "break" | "long_break"
                            ) =>
                              t === "focus"
                                ? "Focus"
                                : t === "long_break"
                                ? "Long Break"
                                : "Short Break";
                            const minutesFor = (
                              t: "focus" | "break" | "long_break"
                            ) =>
                              t === "focus"
                                ? settings.focusDuration
                                : t === "long_break"
                                ? settings.longBreakDuration
                                : settings.shortBreakDuration;
                            const pillFor = (
                              t: "focus" | "break" | "long_break"
                            ) =>
                              t === "focus"
                                ? "bg-blue-500/10 text-blue-300 border-blue-400"
                                : t === "long_break"
                                ? "bg-amber-500/10 text-amber-300 border-amber-400"
                                : "bg-green-500/10 text-green-300 border-green-400";

                            const NowIcon =
                              nowType === "focus" ? (
                                <Target className="w-3 h-3 sm:w-3.5 sm:h-3.5" />
                              ) : (
                                <Coffee className="w-3 h-3 sm:w-3.5 sm:h-3.5" />
                              );
                            const NextIcon =
                              nextType === "focus" ? (
                                <Target className="w-3 h-3 sm:w-3.5 sm:h-3.5" />
                              ) : (
                                <Coffee className="w-3 h-3 sm:w-3.5 sm:h-3.5" />
                              );

                            const stepDisplay = Math.min(
                              currentStepIndex + 1,
                              steps.length
                            );

                            return (
                              <div className="flex flex-col sm:flex-row sm:flex-wrap items-start sm:items-center gap-2 sm:gap-2 mb-3">
                                <div className="flex flex-wrap items-center gap-2">
                                  {nowType && (
                                    <span
                                      className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs sm:text-sm ${pillFor(
                                        nowType
                                      )}`}
                                    >
                                      {NowIcon}
                                      <span className="hidden sm:inline">
                                        Now:{" "}
                                      </span>
                                      {labelFor(nowType)}
                                    </span>
                                  )}
                                  {nextType ? (
                                    <span
                                      className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs sm:text-sm ${pillFor(
                                        nextType
                                      )}`}
                                      title={`${minutesFor(nextType)} minutes`}
                                    >
                                      {NextIcon}
                                      <span className="hidden sm:inline">
                                        Next:{" "}
                                      </span>
                                      {labelFor(nextType)}
                                      <span className="ml-1 text-[10px] sm:text-[11px] text-gray-400">
                                        ({minutesFor(nextType)}m)
                                      </span>
                                    </span>
                                  ) : (
                                    <span className="text-xs sm:text-sm text-gray-500">
                                      No more steps
                                    </span>
                                  )}
                                </div>
                                <span className="text-[10px] sm:text-[11px] text-gray-500 sm:ml-auto">
                                  Step {stepDisplay} of {steps.length}
                                </span>
                              </div>
                            );
                          })()}

                          <div className="mb-4">
                            <div className="flex items-center justify-between mb-2 flex-wrap gap-2">
                              <span className="text-sm sm:text-base font-semibold text-white">
                                Cycle {cycleInGroup} of {cyclesPerLongBreak}
                              </span>
                            </div>

                            {/* Visual steps progress */}
                            <div className="overflow-x-auto pb-1 [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden">
                              <div className="flex items-center gap-1.5 mb-3 min-w-max">
                                {steps.map((step, idx) => {
                                  const isCurrent = idx === currentStepIndex;
                                  const isPast = idx < currentStepIndex;

                                  let bgColor = "bg-gray-800";
                                  let borderColor = "border-gray-700";
                                  let icon = null;

                                  if (step.type === "focus") {
                                    bgColor = isPast
                                      ? "bg-blue-600"
                                      : isCurrent
                                      ? "bg-blue-500"
                                      : "bg-gray-800";
                                    borderColor = isCurrent
                                      ? "border-blue-400"
                                      : "border-gray-700";
                                    icon = <Target className="w-3 h-3" />;
                                  } else if (step.type === "long_break") {
                                    bgColor = isPast
                                      ? "bg-amber-600"
                                      : isCurrent
                                      ? "bg-amber-500"
                                      : "bg-gray-800";
                                    borderColor = isCurrent
                                      ? "border-amber-400"
                                      : "border-gray-700";
                                    icon = <Coffee className="w-3 h-3" />;
                                  } else {
                                    bgColor = isPast
                                      ? "bg-green-600"
                                      : isCurrent
                                      ? "bg-green-500"
                                      : "bg-gray-800";
                                    borderColor = isCurrent
                                      ? "border-green-400"
                                      : "border-gray-700";
                                    icon = <Coffee className="w-3 h-3" />;
                                  }

                                  return (
                                    <div
                                      key={idx}
                                      className="flex items-center gap-1.5"
                                    >
                                      <div
                                        className={`w-7 h-7 sm:w-8 sm:h-8 rounded-lg ${bgColor} border-2 ${borderColor} flex items-center justify-center text-white transition-all ${
                                          isCurrent ? "scale-110 shadow-lg" : ""
                                        }`}
                                      >
                                        {icon}
                                      </div>
                                      {idx < steps.length - 1 && (
                                        <div
                                          className={`w-3 sm:w-4 h-0.5 ${
                                            isPast
                                              ? "bg-green-500"
                                              : "bg-gray-700"
                                          }`}
                                        />
                                      )}
                                    </div>
                                  );
                                })}
                              </div>
                            </div>

                            {/* Current step label */}
                            <div className="text-xs sm:text-sm text-gray-400 text-center sm:text-left">
                              {currentStepIndex < steps.length ? (
                                <>
                                  <span className="hidden sm:inline">
                                    Current:{" "}
                                  </span>
                                  <span className="text-white font-semibold">
                                    {steps[currentStepIndex].type === "focus"
                                      ? "Focus Session"
                                      : steps[currentStepIndex].type ===
                                        "long_break"
                                      ? "Long Break"
                                      : "Short Break"}
                                  </span>
                                </>
                              ) : (
                                <span className="text-green-400 font-semibold">
                                  Día productivo finalizado
                                </span>
                              )}
                            </div>
                          </div>

                          {isReadyForLongBreak && cycleState?.phase !== "idle" && (
                            <div className="mt-2 text-xs text-amber-400 font-semibold text-center">
                              ⭐ Ready for long break!
                            </div>
                          )}
                          
                          {cycleState?.phase === "idle" && 
                           cyclesCompleted > 0 && 
                           cyclesCompleted % cyclesPerLongBreak === 0 && (
                            <div className="mt-2 text-xs sm:text-sm text-green-400 font-semibold text-center">
                              🎉 Día productivo finalizado
                            </div>
                          )}
                        </>
                      );
                    })()}
                  </div>
                </div>
              </section>
            </div>

            {/* Secondary grid */}
            <div className="mt-6 grid grid-cols-1 lg:grid-cols-3 gap-6">
              <section className="bg-gray-900/40 border border-gray-800 rounded-xl p-5">
                <h2 className="text-sm font-semibold text-gray-300 mb-3">
                  Shortcuts
                </h2>
                <ul className="text-sm text-gray-400 space-y-2">
                  <li>
                    <span className="text-gray-500">⌘</span> +{" "}
                    <span className="text-gray-500">Space</span> — Command
                    Palette
                  </li>
                  <li>
                    <span className="text-gray-500">⌘</span> +{" "}
                    <span className="text-gray-500">⇧F</span> — Toggle Focus
                  </li>
                  <li>
                    <span className="text-gray-500">⌘</span> +{" "}
                    <span className="text-gray-500">⇧L</span> — Lock Now
                  </li>
                  <li>
                    <span className="text-gray-500">⌘</span> +{" "}
                    <span className="text-gray-500">P</span> — Abrir Ajustes
                  </li>
                </ul>
              </section>
            </div>
          </div>
        </main>
      </div>
    </div>
  );
}
