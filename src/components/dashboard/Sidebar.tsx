import { useNavigate } from "react-router-dom";
import { LogOut, Home, BarChart3, Settings } from "lucide-react";

export function Sidebar() {
  const navigate = useNavigate();

  const handleLogout = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("logout");
      navigate("/");
    } catch (error) {
      console.error("Error logging out:", error);
    }
  };

  return (
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
  );
}
