import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

interface UserInfo {
  name: string;
  email: string;
  picture: string;
}

export function DashboardHeader() {
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

  const userInitial = useMemo(() => {
    if (!userInfo?.name) return "?";
    return userInfo.name.trim().charAt(0).toUpperCase();
  }, [userInfo?.name]);

  return (
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
  );
}
