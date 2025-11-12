import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import {
  LogOut,
  Home,
  Timer,
  BarChart3,
  Settings,
  Play,
  Coffee,
  Pause,
  Square,
  Target,
  Clock,
  TrendingUp,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useCycleManager } from "../lib/useCycleManager";
import { useCycleState } from "../store";
import { tauriCommands } from "../lib/tauri";
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
  const {
    startFocusSession,
    startBreakSession,
    pauseCycle,
    resumeCycle,
    endSession,
  } = useCycleManager();

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
        const stats = await tauriCommands.getSessionStats(1);
        const today = new Date().toISOString().split("T")[0];
        const todayStat = stats.find((stat) => stat.date === today);
        setTodayStats(todayStat || null);
      } catch (error) {
        console.error("Error loading today stats:", error);
      }
    };

    loadTodayStats();
    // Refresh every minute
    const interval = setInterval(loadTodayStats, 60000);
    return () => clearInterval(interval);
  }, [cycleState?.cycle_count]);

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
              <section className="lg:col-span-2 bg-gray-900/40 border border-gray-800 rounded-xl p-5">
                <div className="flex items-center justify-between mb-4">
                  <h2 className="text-sm font-semibold text-gray-300">
                    Session
                  </h2>
                  <div className="text-xs text-gray-500">
                    {cycleState ? phaseLabel : "Loading…"}
                  </div>
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <div className="text-4xl font-mono font-semibold">
                      {cycleState
                        ? cycleState.phase === "idle"
                          ? "--:--"
                          : formatTime(cycleState.remaining_seconds)
                        : "--:--"}
                    </div>
                    <div className="text-xs text-gray-500 mt-1">
                      {cycleState
                        ? cycleState.is_running
                          ? "Running"
                          : cycleState.phase === "idle"
                          ? "Idle"
                          : "Paused"
                        : "Syncing…"}
                    </div>
                  </div>

                  <div className="flex items-center gap-2">
                    {cycleState?.phase === "idle" && (
                      <>
                        <button
                          onClick={() => startFocusSession()}
                          disabled={!cycleState?.can_start}
                          className="inline-flex items-center gap-2 bg-gray-800 hover:bg-gray-700 text-gray-100 border border-gray-800 rounded-lg px-3 py-2 text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                          <Play className="w-4 h-4" />
                          Start Focus
                        </button>
                        <button
                          onClick={() => startBreakSession(false)}
                          className="inline-flex items-center gap-2 bg-gray-900 hover:bg-gray-800 border border-gray-800 rounded-lg px-3 py-2 text-sm"
                        >
                          <Coffee className="w-4 h-4" />
                          Start Break
                        </button>
                      </>
                    )}

                    {cycleState && cycleState.phase !== "idle" && (
                      <>
                        {!cycleState.is_running ? (
                          <button
                            onClick={() => resumeCycle()}
                            className="inline-flex items-center gap-2 bg-gray-800 hover:bg-gray-700 text-gray-100 border border-gray-800 rounded-lg px-3 py-2 text-sm font-medium"
                          >
                            <Play className="w-4 h-4" />
                            Resume
                          </button>
                        ) : (
                          <button
                            onClick={() => pauseCycle()}
                            className="inline-flex items-center gap-2 bg-gray-900 hover:bg-gray-800 border border-gray-800 rounded-lg px-3 py-2 text-sm"
                          >
                            <Pause className="w-4 h-4" />
                            Pause
                          </button>
                        )}
                        <button
                          onClick={() => endSession(false)}
                          className="inline-flex items-center gap-2 bg-gray-900 hover:bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-red-300 hover:text-red-200"
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
                <div className="grid grid-cols-2 gap-3">
                  <div className="rounded-lg border border-gray-800 bg-gray-900/60 p-4">
                    <div className="flex items-center gap-2 mb-2">
                      <Clock className="w-4 h-4 text-blue-400" />
                      <div className="text-[10px] uppercase tracking-wide text-gray-500">
                        Focus Time
                      </div>
                    </div>
                    <div className="text-2xl font-bold text-white">
                      {todayStats?.focus_minutes || 0}
                      <span className="text-sm font-normal text-gray-400 ml-1">
                        min
                      </span>
                    </div>
                    {cycleState && cycleState.phase === "focus" && (
                      <div className="text-xs text-blue-400 mt-1">
                        Session in progress
                      </div>
                    )}
                    {cycleState &&
                      cycleState.cycle_count > 0 &&
                      cycleState.phase !== "focus" && (
                        <div className="text-xs text-gray-500 mt-1">
                          {cycleState.cycle_count} cycles today
                        </div>
                      )}
                  </div>
                  <div className="rounded-lg border border-gray-800 bg-gray-900/60 p-4">
                    <div className="flex items-center gap-2 mb-2">
                      <Coffee className="w-4 h-4 text-green-400" />
                      <div className="text-[10px] uppercase tracking-wide text-gray-500">
                        Breaks Taken
                      </div>
                    </div>
                    <div className="text-2xl font-bold text-white">
                      {todayStats?.breaks_completed || 0}
                    </div>
                    {cycleState &&
                      cycleState.phase !== "idle" &&
                      cycleState.phase !== "focus" && (
                        <div className="text-xs text-green-400 mt-1">
                          On break now
                        </div>
                      )}
                  </div>
                  <div className="rounded-lg border border-gray-800 bg-gray-900/60 p-4">
                    <div className="flex items-center gap-2 mb-2">
                      <BarChart3 className="w-4 h-4 text-purple-400" />
                      <div className="text-[10px] uppercase tracking-wide text-gray-500">
                        Sessions
                      </div>
                    </div>
                    <div className="text-2xl font-bold text-white">
                      {todayStats?.sessions_completed || 0}
                    </div>
                    {cycleState && cycleState.cycle_count > 0 && (
                      <div className="text-xs text-purple-400 mt-1">
                        {cycleState.cycle_count} cycles completed
                      </div>
                    )}
                  </div>
                  <div className="rounded-lg border border-gray-800 bg-gray-900/60 p-4">
                    <div className="flex items-center gap-2 mb-2">
                      <TrendingUp className="w-4 h-4 text-amber-400" />
                      <div className="text-[10px] uppercase tracking-wide text-gray-500">
                        Next Long Break
                      </div>
                    </div>
                    <div className="text-2xl font-bold text-white">
                      {cycleState && cycleState.cycle_count > 0
                        ? Math.max(0, 4 - (cycleState.cycle_count % 4))
                        : 4}
                    </div>
                    <div className="text-xs text-amber-400 mt-1">
                      {cycleState && cycleState.cycle_count > 0
                        ? cycleState.cycle_count % 4 === 0
                          ? "Ready for long break!"
                          : `${cycleState.cycle_count % 4}/4 cycles`
                        : "Start your first cycle"}
                    </div>
                  </div>
                </div>
              </section>
            </div>

            {/* Secondary grid */}
            <div className="mt-6 grid grid-cols-1 lg:grid-cols-3 gap-6">
              <section className="bg-gray-900/40 border border-gray-800 rounded-xl p-5 lg:col-span-2">
                <h2 className="text-sm font-semibold text-gray-300 mb-3">
                  Activity
                </h2>
                <div className="text-sm text-gray-500">No activity yet.</div>
              </section>
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
