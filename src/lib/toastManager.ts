import { useEffect, useState } from "react";
import type { Toast } from "../types";

type ToastListener = (toasts: Toast[]) => void;

const generateId = () =>
  typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `toast-${Date.now()}-${Math.random().toString(16).slice(2)}`;

class ToastManager {
  private toasts: Toast[] = [];
  private listeners = new Set<ToastListener>();

  private notify() {
    for (const listener of this.listeners) {
      listener([...this.toasts]);
    }
  }

  show(toast: Omit<Toast, "id"> & { id?: string }) {
    const id = toast.id ?? generateId();
    const toastWithId: Toast = { ...toast, id };
    this.toasts = [toastWithId, ...this.toasts];
    this.notify();

    if (toast.duration) {
      window.setTimeout(() => this.dismiss(id), toast.duration);
    }

    return id;
  }

  showError(message: string, options: Partial<Omit<Toast, "type" | "message">> = {}) {
    return this.show({
      type: "error",
      title: options.title ?? "Something went wrong",
      message,
      duration: options.duration ?? 5000,
      action: options.action,
    });
  }

  dismiss(id: string) {
    this.toasts = this.toasts.filter((toast) => toast.id !== id);
    this.notify();
  }

  clear() {
    if (this.toasts.length === 0) return;
    this.toasts = [];
    this.notify();
  }

  subscribe(listener: ToastListener) {
    this.listeners.add(listener);
    listener([...this.toasts]);
    return () => {
      this.listeners.delete(listener);
    };
  }
}

export const toastManager = new ToastManager();

export function useToastState() {
  const [toasts, setToasts] = useState<Toast[]>([]);

  useEffect(() => {
    const unsubscribe = toastManager.subscribe(setToasts);
    return () => unsubscribe();
  }, []);

  return {
    toasts,
    dismiss: (id: string) => toastManager.dismiss(id),
    clear: () => toastManager.clear(),
  };
}

