import { X } from "lucide-react";
import { Z_INDEX } from "../constants/design";
import { useToastState } from "../lib/toastManager";

export function ToastContainer() {
  const { toasts, dismiss } = useToastState();

  if (toasts.length === 0) {
    return null;
  }

  return (
    <div
      className="fixed bottom-6 right-6 flex flex-col gap-3"
      style={{ zIndex: Z_INDEX.toast }}
    >
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`toast ${toast.type}`}
          role="alert"
          aria-live="polite"
        >
          <div className="flex-1">
            <p className="font-semibold text-sm">{toast.title}</p>
            {toast.message && (
              <p className="text-xs text-gray-400 mt-1">{toast.message}</p>
            )}
            {toast.action && (
              <button
                onClick={() => {
                  toast.action?.onClick();
                  dismiss(toast.id);
                }}
                className="mt-2 text-xs font-medium uppercase tracking-wide text-gray-200 hover:text-white transition-colors"
              >
                {toast.action.label}
              </button>
            )}
          </div>
          <button
            onClick={() => dismiss(toast.id)}
            className="text-gray-500 hover:text-gray-300 transition-colors"
            aria-label="Dismiss notification"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      ))}
    </div>
  );
}

