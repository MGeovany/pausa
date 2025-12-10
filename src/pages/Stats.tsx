import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  RefreshCcw,
  Flame,
  BarChart,
  Home,
  Settings,
  BarChart3,
  LogOut,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { tauriCommands } from "../lib/tauri";
import type { SessionStats } from "../types";
import { notificationHelper } from "../lib/notificationHelper";

interface UserInfo {
  name: string;
  email: string;
  picture: string;
}

const RANGE_OPTIONS = [
  { label: "Last 7 days", value: 7 },
  { label: "Last 30 days", value: 30 },
] as const;

type RangeValue = (typeof RANGE_OPTIONS)[number]["value"];

export default function Stats() {
  const navigate = useNavigate();
  const [range, setRange] = useState<RangeValue>(7);
  const [stats, setStats] = useState<SessionStats[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [lastUpdated, setLastUpdated] = useState<string | null>(null);
  const [userInfo, setUserInfo] = useState<UserInfo | null>(null);
  const [avatarError, setAvatarError] = useState(false);
  const [useDemo, setUseDemo] = useState(false);

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

  const userInitial = useMemo(() => {
    if (!userInfo?.name) return "?";
    return userInfo.name.trim().charAt(0).toUpperCase();
  }, [userInfo?.name]);

  const normalizeStat = (stat: any): SessionStats => ({
    date: stat.date,
    focusMinutes: stat.focusMinutes ?? stat.focusMinutes ?? 0,
    breaksCompleted: stat.breaksCompleted ?? stat.breaksCompleted ?? 0,
    sessionsCompleted: stat.sessionsCompleted ?? stat.sessionsCompleted ?? 0,
    evasionAttempts: stat.evasionAttempts ?? stat.evasionAttempts ?? 0,
  });

  const normalizeStats = (raw: SessionStats[]): SessionStats[] =>
    raw.map((item) => normalizeStat(item));

  // Generate deterministic demo data for the last `range` days
  const makeDemoStats = (days: number): SessionStats[] => {
    const today = new Date();
    const demo: SessionStats[] = [];
    for (let i = days - 1; i >= 0; i--) {
      const d = new Date(today);
      d.setDate(d.getDate() - i);
      const dateStr = d.toISOString().split("T")[0];
      // Pattern: higher mid‑week, lower weekends
      const dow = d.getDay(); // 0 Sun ... 6 Sat
      const base = [20, 35, 45, 55, 50, 30, 15][dow]; // minutes
      const noise = (i * 7 + dow) % 8; // deterministic small variation
      const focus = Math.max(0, base + noise);
      const sessions = Math.max(1, Math.round(focus / 25));
      const breaks = Math.max(1, sessions); // 1 break per session
      demo.push(
        normalizeStat({
          date: dateStr,
          focusMinutes: focus,
          breaksCompleted: breaks,
          sessionsCompleted: sessions,
          evasionAttempts: 0,
        })
      );
    }
    return demo;
  };

  useEffect(() => {
    const loadStats = async () => {
      setIsLoading(true);
      try {
        if (useDemo) {
          setStats(makeDemoStats(range));
          setLastUpdated(new Date().toISOString());
          return;
        }
        const data = await tauriCommands.getSessionStats(range);
        setStats(normalizeStats(data));
        setLastUpdated(new Date().toISOString());
      } catch (error) {
        console.error("Failed to load stats:", error);
        notificationHelper.showError(
          "Stats unavailable",
          "We couldn't load your latest stats. Please try again."
        );
      } finally {
        setIsLoading(false);
      }
    };

    loadStats();
  }, [range, useDemo]);

  const aggregates = useMemo(() => {
    if (stats.length === 0) {
      return {
        totalSessions: 0,
        bestDay: null as SessionStats | null,
        focusTrend: 0,
      };
    }

    const totalSessions = stats.reduce(
      (sum, day) => sum + (day.sessionsCompleted || 0),
      0
    );

    const bestDay = stats.reduce((best, current) => {
      if (!best) return current;
      if (!current) return best;
      return (current.focusMinutes || 0) > (best.focusMinutes || 0)
        ? current
        : best;
    }, null as SessionStats | null);

    const half = Math.ceil(stats.length / 2);
    const firstHalf = stats
      .slice(0, half)
      .reduce((sum, day) => sum + (day.sessionsCompleted || 0), 0);
    const secondHalf = stats
      .slice(half)
      .reduce((sum, day) => sum + (day.sessionsCompleted || 0), 0);
    const focusTrend =
      firstHalf === 0
        ? secondHalf > 0
          ? 100
          : 0
        : Math.round(((secondHalf - firstHalf) / firstHalf) * 100);

    return {
      totalSessions,
      bestDay,
      focusTrend,
    };
  }, [stats]);

  // Build a continuous time series for the last `range` days,
  // filling missing days with 0 minutes so the chart always shows all days.
  const daysSeries = useMemo(() => {
    const today = new Date();
    const out: Array<{ date: string; focusMinutes: number }> = [];
    for (let i = range - 1; i >= 0; i--) {
      const d = new Date(today);
      d.setDate(d.getDate() - i);
      const dateStr = d.toISOString().split("T")[0];
      const stat = stats.find((s) => s.date === dateStr);
      out.push({
        date: dateStr,
        focusMinutes: stat?.focusMinutes ?? 0,
      });
    }
    return out;
  }, [stats, range]);

  const maxFocusMinutes = useMemo(() => {
    const values = daysSeries.map((d) => d.focusMinutes);
    const max = values.length > 0 ? Math.max(...values) : 0;
    return Math.max(max, 1);
  }, [daysSeries]);

  const handleRangeChange = (value: RangeValue) => {
    setRange(value);
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
                className="p-2 rounded-lg hover:bg-gray-800"
                title="Home"
              >
                <Home className="w-5 h-5" />
              </button>
              <button
                onClick={() => navigate("/stats")}
                className="p-2 rounded-lg bg-gray-800"
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
            <header className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between mb-8">
              <div>
                <h1 className="text-3xl font-semibold text-white">
                  Statistics
                </h1>
                <p className="text-sm text-gray-400 mt-2">
                  Track your progress and uncover personal focus patterns.
                </p>
              </div>

              <div className="flex flex-wrap items-center gap-3">
                <div className="flex rounded-full border border-gray-800 bg-gray-900/80 p-1">
                  {RANGE_OPTIONS.map((option) => (
                    <button
                      key={option.value}
                      onClick={() => handleRangeChange(option.value)}
                      className={`px-4 py-2 text-sm font-medium rounded-full transition-colors ${
                        range === option.value
                          ? "bg-blue-600 text-white shadow-lg"
                          : "text-gray-400 hover:text-white"
                      }`}
                    >
                      {option.label}
                    </button>
                  ))}
                </div>
                <button
                  onClick={() => setUseDemo((prev) => !prev)}
                  disabled={isLoading}
                  className={`inline-flex items-center gap-2 rounded-xl border px-4 py-2 text-sm transition-colors ${
                    useDemo
                      ? "border-amber-500/40 bg-amber-500/10 text-amber-300 hover:bg-amber-500/20"
                      : "border-gray-800 bg-gray-900/60 text-gray-300 hover:text-white hover:bg-gray-900"
                  } disabled:opacity-50`}
                  title={useDemo ? "Using demo data" : "Load demo data"}
                >
                  {useDemo ? "Demo on" : "Demo data"}
                </button>
                <button
                  onClick={() => setRange((prev) => prev)} // retrigger load
                  disabled={isLoading}
                  className="inline-flex items-center gap-2 rounded-xl border border-gray-800 bg-gray-900/60 px-4 py-2 text-sm text-gray-300 hover:text-white hover:bg-gray-900 transition-colors disabled:opacity-50"
                >
                  <RefreshCcw
                    className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`}
                  />
                  Actualizar
                </button>
              </div>
            </header>

            <section className="mt-8 grid gap-4 md:grid-cols-2">
              <StatCard
                icon={<Flame className="h-5 w-5" />}
                label="Sessions completed"
                value={
                  isNaN(aggregates.totalSessions)
                    ? "0"
                    : aggregates.totalSessions.toString()
                }
                helper={
                  aggregates.focusTrend >= 0 && !isNaN(aggregates.focusTrend)
                    ? `▲ ${aggregates.focusTrend}% vs previous period`
                    : aggregates.focusTrend < 0 && !isNaN(aggregates.focusTrend)
                    ? `▼ ${Math.abs(aggregates.focusTrend)}% vs previous period`
                    : "No comparison available"
                }
                helperTone={
                  aggregates.focusTrend >= 0 && !isNaN(aggregates.focusTrend)
                    ? "positive"
                    : aggregates.focusTrend < 0 && !isNaN(aggregates.focusTrend)
                    ? "negative"
                    : undefined
                }
              />
              <StatCard
                icon={<BarChart className="h-5 w-5" />}
                label="Best day"
                value={
                  aggregates.bestDay && aggregates.bestDay.focusMinutes > 0
                    ? new Date(aggregates.bestDay.date).toLocaleDateString(
                        undefined,
                        {
                          weekday: "short",
                          day: "numeric",
                          month: "short",
                        }
                      )
                    : "No data"
                }
                helper={
                  aggregates.bestDay && aggregates.bestDay.focusMinutes > 0
                    ? `${aggregates.bestDay.focusMinutes} focus minutes`
                    : "Start a session to populate this insight"
                }
              />
            </section>

            <section className="mt-10 grid gap-6 lg:grid-cols-3 min-w-0">
              <div className="lg:col-span-2 rounded-2xl border border-gray-800 bg-gray-900/60 p-6 min-w-0">
                <div className="flex items-center justify-between">
                  <h2 className="text-lg font-semibold text-white">
                    Focus pace (last {range} days)
                  </h2>
                  <span className="text-xs text-gray-500">
                    Each bar represents the focus minutes for that day
                  </span>
                </div>
                <div className="mt-6 flex items-end gap-3 overflow-x-auto pb-3">
                  {daysSeries.map((day) => {
                    const barHeight = Math.max(
                      6,
                      Math.round((day.focusMinutes / maxFocusMinutes) * 180)
                    );
                    return (
                      <div
                        key={day.date}
                        className="flex flex-col items-center text-xs text-gray-400"
                      >
                        <div
                          className="w-8 rounded-t-lg bg-gradient-to-t from-blue-600 to-blue-400 shadow-lg shadow-blue-500/20 transition-all hover:to-blue-300"
                          style={{ height: `${barHeight}px` }}
                        />
                        <span className="mt-2 font-medium text-gray-300">
                          {formatDayLabel(day.date)}
                        </span>
                        <span>{day.focusMinutes}m</span>
                      </div>
                    );
                  })}
                  {daysSeries.every((d) => d.focusMinutes === 0) && (
                    <div className="flex h-40 w-full items-center justify-center text-sm text-gray-500">
                      No sessions recorded yet.
                    </div>
                  )}
                </div>
              </div>

              <div className="rounded-2xl border border-gray-800 bg-gray-900/60 p-6 min-w-0">
                <h2 className="text-lg font-semibold text-white mb-1">
                  Energy map
                </h2>
                <p className="text-xs text-gray-500 mb-4">
                  GitHub-style monthly calendar — darker colors mean more focus
                  time.
                </p>
                {/* GitHub-style monthly contribution calendar (last 12 months) */}
                <div className="mt-4 overflow-x-auto w-full [scrollbar-width:thin] [scrollbar-color:rgb(75,85,99)_rgb(31,41,55)] [-ms-overflow-style:none] [&::-webkit-scrollbar]:h-2 [&::-webkit-scrollbar-track]:bg-gray-800/50 [&::-webkit-scrollbar-track]:rounded-full [&::-webkit-scrollbar-thumb]:bg-gray-600 [&::-webkit-scrollbar-thumb]:rounded-full [&::-webkit-scrollbar-thumb]:hover:bg-gray-500">
                  {(() => {
                    // Build continuous day series for last 365 days using energyStats (or demo)
                    const end = new Date();
                    const start = new Date(end);
                    start.setDate(end.getDate() - 364);
                    // align start to Sunday to begin weeks properly
                    while (start.getDay() !== 0) {
                      start.setDate(start.getDate() - 1);
                    }
                    const dayMs = 24 * 60 * 60 * 1000;
                    const days: Array<{ date: Date; minutes: number }> = [];
                    const lookup = new Map<string, number>();
                    stats.forEach((s) =>
                      lookup.set(s.date, s.focusMinutes || 0)
                    );

                    for (
                      let t = start.getTime();
                      t <= end.getTime();
                      t += dayMs
                    ) {
                      const d = new Date(t);
                      const dateStr = d.toISOString().split("T")[0];
                      const minutes = lookup.get(dateStr) ?? 0;
                      days.push({ date: d, minutes });
                    }

                    // Group into weeks (columns)
                    const weeks: Array<Array<{ date: Date; minutes: number }>> =
                      [];
                    for (let i = 0; i < days.length; i += 7) {
                      weeks.push(days.slice(i, i + 7));
                    }

                    const monthLabel = (d: Date) =>
                      d.toLocaleString(undefined, { month: "short" });

                    const cellFor = (minutes: number) => {
                      const intensity =
                        minutes === 0
                          ? 0
                          : minutes < 15
                          ? 1
                          : minutes < 30
                          ? 2
                          : minutes < 60
                          ? 3
                          : 4;
                      const bgColor =
                        intensity === 0
                          ? "bg-gray-800/60 border-gray-800"
                          : intensity === 1
                          ? "bg-green-900/80 border-green-800"
                          : intensity === 2
                          ? "bg-green-800/80 border-green-700"
                          : intensity === 3
                          ? "bg-green-600/90 border-green-600"
                          : "bg-green-500 border-green-500";
                      return bgColor;
                    };

                    return (
                      <div className="inline-block min-w-max">
                        {/* Month labels */}
                        <div className="flex items-center gap-1 ml-6 mb-2">
                          {weeks.map((week, idx) => {
                            const first = week[0]?.date;
                            const prev = weeks[idx - 1]?.[0]?.date;
                            const show =
                              idx === 0 ||
                              (prev &&
                                first &&
                                first.getMonth() !== prev.getMonth());
                            return (
                              <div
                                key={`m-${idx}`}
                                className="w-3 text-[10px] text-gray-500"
                              >
                                {show && first ? monthLabel(first) : ""}
                              </div>
                            );
                          })}
                        </div>
                        {/* Weekday labels + grid */}
                        <div className="flex gap-1">
                          {/* Weekday labels */}
                          <div className="flex flex-col justify-between mr-2">
                            {["Sun", "Tue", "Thu"].map((d) => (
                              <div
                                key={d}
                                className="h-3 text-[10px] text-gray-500"
                              >
                                {d}
                              </div>
                            ))}
                          </div>
                          {/* Weeks */}
                          <div className="flex gap-1">
                            {weeks.map((week, wIdx) => (
                              <div
                                key={`w-${wIdx}`}
                                className="flex flex-col gap-1"
                              >
                                {week.map((day, dIdx) => (
                                  <div
                                    key={`d-${wIdx}-${dIdx}`}
                                    className={`h-3 w-3 rounded-sm border ${cellFor(
                                      day.minutes
                                    )} transition-all hover:scale-110`}
                                    title={`${day.date.toLocaleDateString(
                                      undefined,
                                      {
                                        weekday: "long",
                                        day: "numeric",
                                        month: "short",
                                      }
                                    )} • ${day.minutes} min`}
                                  />
                                ))}
                              </div>
                            ))}
                          </div>
                        </div>
                        {/* Legend */}
                        <div className="flex items-center justify-end gap-2 mt-4 text-xs">
                          <span className="text-gray-500">Less</span>
                          <div className="flex gap-1">
                            <div className="h-3 w-3 rounded-sm bg-gray-800/60 border border-gray-800" />
                            <div className="h-3 w-3 rounded-sm bg-green-900/80 border border-green-800" />
                            <div className="h-3 w-3 rounded-sm bg-green-800/80 border border-green-700" />
                            <div className="h-3 w-3 rounded-sm bg-green-600/90 border border-green-600" />
                            <div className="h-3 w-3 rounded-sm bg-green-500 border border-green-500" />
                          </div>
                          <span className="text-gray-200">More</span>
                        </div>
                      </div>
                    );
                  })()}
                </div>
              </div>
            </section>

            <section className="mt-10 rounded-2xl border border-gray-800 bg-gray-900/60 p-6">
              <div className="flex items-center justify-between">
                <div>
                  <h2 className="text-lg font-semibold text-white">
                    Detailed history
                  </h2>
                  <p className="text-xs text-gray-500">
                    Sessions recorded during the selected range.
                  </p>
                </div>
                {lastUpdated && (
                  <span className="text-xs text-gray-500">
                    Updated {new Date(lastUpdated).toLocaleTimeString()}
                  </span>
                )}
              </div>
              <div className="mt-4 overflow-hidden rounded-xl border border-gray-800">
                <table className="min-w-full divide-y divide-gray-800 text-sm text-gray-300">
                  <thead className="bg-gray-900/80">
                    <tr>
                      <th className="px-4 py-3 text-left font-medium text-gray-400">
                        Date
                      </th>
                      <th className="px-4 py-3 text-left font-medium text-gray-400">
                        Focus Time
                      </th>
                      <th className="px-4 py-3 text-left font-medium text-gray-400">
                        Sessions
                      </th>
                      <th className="px-4 py-3 text-left font-medium text-gray-400">
                        Avg Session
                      </th>
                      <th className="px-4 py-3 text-left font-medium text-gray-400">
                        Completion Rate
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-gray-900">
                    {stats.map((day) => {
                      const avgSession =
                        day.sessionsCompleted > 0
                          ? Math.round(day.focusMinutes / day.sessionsCompleted)
                          : 0;
                      const completionRate =
                        day.sessionsCompleted > 0
                          ? Math.round(
                              (day.sessionsCompleted /
                                (day.sessionsCompleted +
                                  (day.evasionAttempts || 0))) *
                                100
                            )
                          : 100;

                      return (
                        <tr
                          key={`row-${day.date}`}
                          className="hover:bg-gray-900/70"
                        >
                          <td className="px-4 py-3 font-medium text-white">
                            {new Date(day.date).toLocaleDateString(undefined, {
                              weekday: "short",
                              day: "numeric",
                              month: "short",
                            })}
                          </td>
                          <td className="px-4 py-3">
                            <span className="font-semibold">
                              {day.focusMinutes}
                            </span>
                            <span className="text-gray-500 ml-1">min</span>
                          </td>
                          <td className="px-4 py-3">
                            <span className="font-semibold">
                              {day.sessionsCompleted}
                            </span>
                            <span className="text-gray-500 ml-1">sessions</span>
                          </td>
                          <td className="px-4 py-3">
                            {avgSession > 0 ? (
                              <>
                                <span className="font-semibold">
                                  {avgSession}
                                </span>
                                <span className="text-gray-500 ml-1">
                                  min/session
                                </span>
                              </>
                            ) : (
                              <span className="text-gray-500">-</span>
                            )}
                          </td>
                          <td className="px-4 py-3">
                            <span
                              className={`font-semibold ${
                                completionRate >= 90
                                  ? "text-green-400"
                                  : completionRate >= 70
                                  ? "text-yellow-400"
                                  : "text-red-400"
                              }`}
                            >
                              {completionRate}%
                            </span>
                          </td>
                        </tr>
                      );
                    })}
                    {stats.length === 0 && (
                      <tr>
                        <td
                          className="px-4 py-8 text-center text-sm text-gray-500"
                          colSpan={5}
                        >
                          There are no records for this range yet.
                        </td>
                      </tr>
                    )}
                  </tbody>
                </table>
              </div>
            </section>
          </div>
        </main>
      </div>
    </div>
  );
}

interface StatCardProps {
  icon: React.ReactNode;
  label: string;
  value: string;
  helper?: string;
  helperTone?: "positive" | "negative";
}

function StatCard({ icon, label, value, helper, helperTone }: StatCardProps) {
  return (
    <div className="rounded-2xl border border-gray-800 bg-gray-900/60 p-5 shadow-[0_10px_40px_-20px_rgba(78,142,247,0.5)]">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3 text-sm font-semibold uppercase tracking-wide text-gray-500">
          <span className="flex h-9 w-9 items-center justify-center rounded-xl bg-blue-500/20 text-blue-300">
            {icon}
          </span>
          {label}
        </div>
        <div className="text-2xl font-semibold text-white">{value}</div>
      </div>
      {helper && (
        <p
          className={`mt-3 text-xs ${
            helperTone === "positive"
              ? "text-green-400"
              : helperTone === "negative"
              ? "text-red-400"
              : "text-gray-500"
          }`}
        >
          {helper}
        </p>
      )}
    </div>
  );
}

function formatDayLabel(dateString: string) {
  const date = new Date(dateString);
  return date.toLocaleDateString(undefined, { weekday: "short" });
}
