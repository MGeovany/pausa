import { useEffect, useRef, useState, type ReactNode } from "react";
import { X, BarChart3, Clock, Coffee } from "lucide-react";
import { tauriCommands } from "../lib/tauri";
import { toastManager } from "../lib/toastManager";
import { useAppStore, useStats } from "../store";

export function StatsModal() {
  const isStatsOpen = useAppStore((state) => state.isStatsOpen);
  const hideStats = useAppStore((state) => state.hideStats);
  const setStats = useAppStore((state) => state.setStats);
  const stats = useStats();
  const [isLoading, setIsLoading] = useState(false);
  const hasRequestedStats = useRef(false);

  useEffect(() => {
    if (!isStatsOpen) {
      hasRequestedStats.current = false;
      return;
    }

    if (hasRequestedStats.current) {
      return;
    }

    hasRequestedStats.current = true;
    let isCancelled = false;

    const fetchStats = async () => {
      setIsLoading(true);
      try {
        const data = await tauriCommands.getSessionStats(7);
        if (!isCancelled) {
          setStats(data);
        }
      } catch (error) {
        console.error("Failed to load session stats:", error);
        toastManager.showError(
          "We couldn't load your recent stats. Please try again later.",
          { title: "Stats Unavailable" }
        );
      } finally {
        if (!isCancelled) {
          setIsLoading(false);
        }
      }
    };

    fetchStats();

    return () => {
      isCancelled = true;
    };
  }, [isStatsOpen, setStats]);

  if (!isStatsOpen) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
      <div className="w-full max-w-3xl rounded-2xl border border-gray-800 bg-gray-950/95 p-6 shadow-2xl">
        <div className="flex items-center justify-between border-b border-gray-800 pb-4">
          <div className="flex items-center gap-2">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-blue-500/20 text-blue-300">
              <BarChart3 className="h-5 w-5" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-white">Focus Stats</h2>
              <p className="text-sm text-gray-400">
                Overview of your last 7 days of focus and breaks
              </p>
            </div>
          </div>
          <button
            onClick={hideStats}
            className="rounded-full border border-gray-700 bg-gray-800/60 p-2 text-gray-400 transition-colors hover:text-white"
            aria-label="Close stats"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="mt-6 space-y-6">
          {isLoading ? (
            <div className="flex h-48 items-center justify-center text-sm text-gray-400">
              Loading your progress…
            </div>
          ) : stats.length === 0 ? (
            <div className="flex h-48 flex-col items-center justify-center gap-2 text-sm text-gray-400">
              <p>No focus sessions recorded yet.</p>
              <p>Start a session to see statistics here.</p>
            </div>
          ) : (
            <>
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
                <StatCard
                  icon={<Clock className="h-4 w-4" />}
                  label="Total Focus Minutes"
                  value={`${stats.reduce(
                    (total, day) => total + day.focus_minutes,
                    0
                  )}`}
                />
                <StatCard
                  icon={<Coffee className="h-4 w-4" />}
                  label="Breaks Completed"
                  value={`${stats.reduce(
                    (total, day) => total + day.breaks_completed,
                    0
                  )}`}
                />
                <StatCard
                  icon={<BarChart3 className="h-4 w-4" />}
                  label="Sessions Completed"
                  value={`${stats.reduce(
                    (total, day) => total + day.sessions_completed,
                    0
                  )}`}
                />
              </div>

              <div className="overflow-hidden rounded-xl border border-gray-800">
                <table className="min-w-full divide-y divide-gray-800 text-sm text-gray-300">
                  <thead className="bg-gray-900">
                    <tr>
                      <th className="px-4 py-3 text-left font-medium text-gray-400">
                        Date
                      </th>
                      <th className="px-4 py-3 text-left font-medium text-gray-400">
                        Focus Minutes
                      </th>
                      <th className="px-4 py-3 text-left font-medium text-gray-400">
                        Breaks
                      </th>
                      <th className="px-4 py-3 text-left font-medium text-gray-400">
                        Sessions
                      </th>
                      <th className="px-4 py-3 text-left font-medium text-gray-400">
                        Evasion Attempts
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-gray-900">
                    {stats.map((day) => (
                      <tr key={day.date} className="hover:bg-gray-900/40">
                        <td className="px-4 py-3 font-medium text-white">
                          {new Date(day.date).toLocaleDateString()}
                        </td>
                        <td className="px-4 py-3">{day.focus_minutes} min</td>
                        <td className="px-4 py-3">{day.breaks_completed}</td>
                        <td className="px-4 py-3">{day.sessions_completed}</td>
                        <td className="px-4 py-3">
                          {day.evasion_attempts ?? 0}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

interface StatCardProps {
  icon: ReactNode;
  label: string;
  value: string;
}

function StatCard({ icon, label, value }: StatCardProps) {
  return (
    <div className="rounded-xl border border-gray-800 bg-gray-900/60 p-4">
      <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-gray-500">
        {icon}
        {label}
      </div>
      <div className="mt-2 text-2xl font-semibold text-white">{value}</div>
    </div>
  );
}
