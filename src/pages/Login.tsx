import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export default function Login() {
  const [status, setStatus] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  console.log("🔵 Login component rendered");

  useEffect(() => {
    console.log("🎧 Setting up event listeners...");

    const unlistenSuccess = listen("auth:success", (event) => {
      console.log("🎉 Auth success event received!", event);
      setStatus("Login successful! Redirecting...");
      setIsLoading(false);
      setTimeout(() => {
        console.log("🚀 Redirecting to dashboard...");
        window.location.hash = "#/dashboard";
      }, 1000);
    });

    const unlistenError = listen("auth:error", (e) => {
      console.error("❌ Auth error event:", e.payload);
      setStatus("Error: " + String(e.payload));
      setIsLoading(false);
    });

    console.log("✅ Event listeners set up successfully");

    // Check if tokens exist and redirect automatically
    const checkTokens = async () => {
      try {
        const tokens = await invoke("read_tokens");
        if (tokens) {
          console.log("✅ Tokens found, redirecting to dashboard...");
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
      console.log("🧹 Cleaning up event listeners...");
      clearInterval(interval);
      unlistenSuccess.then((fn) => fn());
      unlistenError.then((fn) => fn());
    };
  }, []);

  const handleLogin = async () => {
    console.log("🔐 Login button clicked");
    setIsLoading(true);
    setStatus("Opening browser...");

    try {
      console.log("📞 Calling Tauri command: login_with_google");
      const result = await invoke("login_with_google");
      console.log("✅ Tauri command result:", result);
      setStatus("Browser opened, waiting for authentication...");
    } catch (error) {
      console.error("❌ Error calling login_with_google:", error);
      setStatus("Error: " + String(error));
      setIsLoading(false);
    }
  };

  return (
    <div className="flex items-center justify-center min-h-screen bg-white">
      <div className="w-full max-w-sm px-8">
        {/* Logo */}
        <div className="mb-12">
          <div className="w-10 h-10 bg-black rounded-lg flex items-center justify-center">
            <span className="text-white text-xl font-bold">P</span>
          </div>
        </div>

        {/* Heading */}
        <div className="mb-10">
          <h1 className="text-3xl font-semibold text-black mb-2">
            Welcome back
          </h1>
          <p className="text-gray-600 text-sm">Sign in to continue to Pausa</p>
        </div>

        {/* Login Button */}
        <div className="space-y-4">
          <button
            onClick={handleLogin}
            disabled={isLoading}
            className="w-full bg-black text-white py-3 px-4 rounded-lg transition-all hover:bg-gray-900 disabled:opacity-50 disabled:cursor-not-allowed font-medium text-sm"
          >
            {isLoading ? "Signing in..." : "Continue with Google"}
          </button>

          {status && (
            <div className="text-center py-3 px-4 rounded-lg border border-gray-200">
              <p className="text-sm text-gray-700">{status}</p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="mt-12 text-center">
          <p className="text-xs text-gray-500">
            By continuing, you agree to our{" "}
            <a href="#" className="text-black hover:underline">
              Terms of Service
            </a>{" "}
            and{" "}
            <a href="#" className="text-black hover:underline">
              Privacy Policy
            </a>
          </p>
        </div>
      </div>
    </div>
  );
}
