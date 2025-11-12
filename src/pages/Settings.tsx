import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  Home,
  Settings as SettingsIcon,
  BarChart3,
  LogOut,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { Settings as SettingsPanel } from "../components/Settings";

interface UserInfo {
  name: string;
  email: string;
  picture: string;
}

export default function Settings() {
  const navigate = useNavigate();
  const [userInfo, setUserInfo] = useState<UserInfo | null>(null);
  const [avatarError, setAvatarError] = useState(false);

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
                className="p-2 rounded-lg hover:bg-gray-800"
                title="Stats"
              >
                <BarChart3 className="w-5 h-5" />
              </button>
              <button
                onClick={() => navigate("/settings")}
                className="p-2 rounded-lg bg-gray-800"
                title="Settings"
              >
                <SettingsIcon className="w-5 h-5" />
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
            <SettingsPanel />
          </div>
        </main>
      </div>
    </div>
  );
}
