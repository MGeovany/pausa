import { useNavigate } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { Settings as SettingsPanel } from "../components/Settings";

export default function Settings() {
  const navigate = useNavigate();

  return (
    <div className="min-h-screen bg-gray-950 text-gray-100">
      <div className="mx-auto w-full max-w-5xl px-6 py-8">
        <button
          onClick={() => navigate(-1)}
          className="inline-flex items-center gap-2 text-sm text-gray-400 hover:text-white transition-colors"
        >
          <ArrowLeft className="h-4 w-4" />
          Back
        </button>

        <div className="mt-6">
          <SettingsPanel />
        </div>
      </div>
    </div>
  );
}

