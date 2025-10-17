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

        {/* Stats Grid */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          <div className="rounded-lg p-6 border border-gray-200 hover:border-gray-300 transition-all bg-white">
            <div className="flex items-center justify-between mb-4">
              <div className="w-10 h-10 bg-black rounded-lg flex items-center justify-center">
                <Clock size={20} className="text-white" />
              </div>
            </div>
            <h3 className="text-3xl font-bold text-black mb-1">24h</h3>
            <p className="text-sm text-gray-600 font-medium">Total time</p>
          </div>

          <div className="rounded-lg p-6 border border-gray-200 hover:border-gray-300 transition-all bg-white">
            <div className="flex items-center justify-between mb-4">
              <div className="w-10 h-10 bg-black rounded-lg flex items-center justify-center">
                <Target size={20} className="text-white" />
              </div>
            </div>
            <h3 className="text-3xl font-bold text-black mb-1">12</h3>
            <p className="text-sm text-gray-600 font-medium">
              Sessions completed
            </p>
          </div>

          <div className="rounded-lg p-6 border border-gray-200 hover:border-gray-300 transition-all bg-white">
            <div className="flex items-center justify-between mb-4">
              <div className="w-10 h-10 bg-black rounded-lg flex items-center justify-center">
                <TrendingUp size={20} className="text-white" />
              </div>
            </div>
            <h3 className="text-3xl font-bold text-black mb-1">85%</h3>
            <p className="text-sm text-gray-600 font-medium">Efficiency</p>
          </div>
        </div>

        {/* Main Content */}
        <div className="rounded-lg p-8 border border-gray-200 bg-white">
          <h2 className="text-xl font-semibold text-black mb-4">
            Recent activity
          </h2>
          <p className="text-gray-600">
            Your activity will appear here when you start using Pausa.
          </p>
        </div>
      </div>
    </div>
  );
}
