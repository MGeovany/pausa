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
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useCycleManager } from "../lib/useCycleManager";
import { useCycleState } from "../store";

interface UserInfo {
  name: string;
  email: string;
  picture: string;
}

export default function Dashboard() {
  const navigate = useNavigate();
  const [userInfo, setUserInfo] = useState<UserInfo | null>(null);
  const [avatarError, setAvatarError] = useState(false);
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
              <button className="p-2 rounded-lg hover:bg-gray-800" title="Home">
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
          <div className="px-6 md:px-8 py-6">
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

              {/* Compact Stats */}
              <section className="bg-gray-900/40 border border-gray-800 rounded-xl p-5">
                <h2 className="text-sm font-semibold text-gray-300 mb-4">
                  Today
                </h2>
                <div className="grid grid-cols-3 gap-3">
                  <div className="rounded-lg border border-gray-800 bg-gray-900/60 p-3">
                    <div className="text-[10px] uppercase tracking-wide text-gray-500 mb-1">
                      Phase
                    </div>
                    <div className="text-sm">{phaseLabel}</div>
                  </div>
                  <div className="rounded-lg border border-gray-800 bg-gray-900/60 p-3">
                    <div className="text-[10px] uppercase tracking-wide text-gray-500 mb-1">
                      Remaining
                    </div>
                    <div className="text-sm">
                      {cycleState && cycleState.phase !== "idle"
                        ? formatTime(cycleState.remaining_seconds)
                        : "--:--"}
                    </div>
                  </div>
                  <div className="rounded-lg border border-gray-800 bg-gray-900/60 p-3">
                    <div className="text-[10px] uppercase tracking-wide text-gray-500 mb-1">
                      Cycles
                    </div>
                    <div className="text-sm">
                      {cycleState ? cycleState.cycle_count : 0}
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
