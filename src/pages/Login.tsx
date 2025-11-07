import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ArrowRightIcon } from "lucide-react";

export default function Login() {
  const [status, setStatus] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    const unlistenSuccess = listen("auth:success", () => {
      setStatus("Login successful! Redirecting...");
      setIsLoading(false);
      setTimeout(() => {
        window.location.hash = "#/dashboard";
      }, 1000);
    });

    const unlistenError = listen("auth:error", (e) => {
      setStatus("Error: " + String(e.payload));
      setIsLoading(false);
    });

    // Check if tokens exist
    const checkTokens = async () => {
      try {
        const tokens = await invoke("read_tokens");
        if (tokens) {
          window.location.hash = "#/dashboard";
        }
      } catch (error) {
        console.error("Error checking tokens:", error);
      }
    };

    // Check tokens every 2 seconds
    const interval = setInterval(checkTokens, 2000);

    // Cleanup function
    return () => {
      clearInterval(interval);
      unlistenSuccess.then((fn) => fn());
      unlistenError.then((fn) => fn());
    };
  }, []);

  const handleLogin = async () => {
    setIsLoading(true);
    setStatus("Opening browser...");

    try {
      await invoke("login_with_google");
      setStatus("Browser opened, waiting for authentication...");
    } catch (error) {
      console.error("Error calling login_with_google:", error);
      setStatus("Error: " + String(error));
      setIsLoading(false);
    }
  };

  return (
    <div
      className="min-h-screen bg-gradient-to-br from-zinc-900 via-zinc-800 to-zinc-900 flex items-center justify-center p-4 relative overflow-hidden rounded-md"
      style={{
        fontFamily: "BBH Sans Bartle",
        fontWeight: 400,
        fontStyle: "normal",
      }}
    >
      <div className="absolute top-0 right-0 w-[600px] h-[600px] bg-white/10 rounded-full blur-3xl translate-x-1/2 -translate-y-1/2" />
      <div className="absolute bottom-0 left-0 w-[600px] h-[600px] bg-white/10 rounded-full blur-3xl -translate-x-1/2 translate-y-1/2" />

      <div className="w-full max-w-md relative z-10">
        <div className="mb-6 max-w-md h-[400px] w-full flex flex-col items-center justify-center">
          <p className="text-4xl bg-[#1f1f20] font-bold rounded-xl py-2 px-4 mb-4">
            P
          </p>
          <h1 className="text-5xl font-bold text-center mb-4">
            Welcome to Pausa
          </h1>
          <p className="text-sm text-center opacity-50 mb-8">
            Focus. Breathe. Begin your next session.
          </p>
          
          {status && (
            <p className="text-sm text-center text-gray-300 mb-4">
              {status}
            </p>
          )}
          
          <a
            onClick={handleLogin}
            className={`w-fit text-sm text-center flex items-center justify-center gap-[2px] cursor-pointer transition-opacity duration-700 ${
              isLoading ? "opacity-50 cursor-not-allowed" : "opacity-50 hover:opacity-100"
            }`}
            style={{
              fontFamily: "Poppins",
              fontWeight: 500,
              fontStyle: "normal",
            }}
          >
            {isLoading ? (
              <>
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                <span>Connecting...</span>
              </>
            ) : (
              <>
                <span>Continue with Google</span>
                <ArrowRightIcon className="w-4 h-4 mt-[1px]" strokeWidth={3} />
              </>
            )}
          </a>
          
          {/* Development button to reset onboarding */}
          {process.env.NODE_ENV === 'development' && (
            <button
              onClick={async () => {
                try {
                  await invoke("reset_onboarding_for_testing");
                  setStatus("Onboarding reset! Refresh the page to test onboarding flow.");
                } catch (error) {
                  console.error("Error resetting onboarding:", error);
                  setStatus("Error resetting onboarding: " + String(error));
                }
              }}
              className="mt-4 text-xs text-red-400 hover:text-red-300 underline"
            >
              [DEV] Reset Onboarding
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
