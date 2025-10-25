import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { LogOut, Clock, Target, TrendingUp } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

interface UserInfo {
  name: string;
  email: string;
  picture: string;
}

export default function Dashboard() {
  const navigate = useNavigate();
  const [userInfo, setUserInfo] = useState<UserInfo | null>(null);

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

  const handleLogout = async () => {
    try {
      await invoke("logout");
      navigate("/");
    } catch (error) {
      console.error("Error logging out:", error);
    }
  };

  return (
    <div className="min-h-screen bg-white" style={{ fontFamily: "Cal Sans" }}>
      <div className="max-w-7xl mx-auto px-8 py-12">
        {/* Header */}
        <div className="flex items-center justify-between mb-12">
          <div>
            <h1 className="text-3xl font-semibold text-black mb-1">
              Dashboard
            </h1>
            <p className="text-sm text-gray-600">Your productivity summary</p>
          </div>
          <div className="flex items-center gap-6">
            {userInfo && (
              <div className="flex items-center gap-4">
                <img
                  src={userInfo.picture}
                  alt={userInfo.name}
                  className="w-12 h-12 rounded-full border-2 border-gray-200"
                />
                <div className="text-left">
                  <p className="text-sm font-semibold text-black leading-tight">
                    {userInfo.name}
                  </p>
                  <p className="text-xs text-gray-500 leading-tight">
                    {userInfo.email}
                  </p>
                </div>
              </div>
            )}
            <button
              onClick={handleLogout}
              className="flex items-center gap-2 px-5 py-2.5 bg-black text-white rounded-lg hover:bg-gray-900 transition-all text-sm font-medium"
            >
              <LogOut size={16} />
              Logout
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
