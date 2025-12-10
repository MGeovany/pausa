import {
  sendNotification,
  isPermissionGranted,
  requestPermission,
} from "@tauri-apps/plugin-notification";

/**
 * Helper for showing system notifications
 * Replaces toastManager with native OS notifications
 */
class NotificationHelper {
  /**
   * Ensure notification permission is granted
   */
  private async ensurePermission(): Promise<boolean> {
    try {
      let permissionGranted = await isPermissionGranted();
      if (!permissionGranted) {
        const permission = await requestPermission();
        permissionGranted = permission === "granted";
      }
      return permissionGranted;
    } catch (error) {
      console.error("Failed to check/request notification permission:", error);
      return false;
    }
  }

  /**
   * Show an error notification
   */
  async showError(title: string, body?: string) {
    try {
      const hasPermission = await this.ensurePermission();
      if (hasPermission) {
        await sendNotification({
          title,
          body: body || title,
        });
      }
    } catch (error) {
      console.error("Failed to show notification:", error);
    }
  }

  /**
   * Show a success notification
   */
  async showSuccess(title: string, body?: string) {
    try {
      const hasPermission = await this.ensurePermission();
      if (hasPermission) {
        await sendNotification({
          title,
          body: body || title,
        });
      }
    } catch (error) {
      console.error("Failed to show notification:", error);
    }
  }

  /**
   * Show an info notification
   */
  async showInfo(title: string, body?: string) {
    try {
      const hasPermission = await this.ensurePermission();
      if (hasPermission) {
        await sendNotification({
          title,
          body: body || title,
        });
      }
    } catch (error) {
      console.error("Failed to show notification:", error);
    }
  }

  /**
   * Show a warning notification
   */
  async showWarning(title: string, body?: string) {
    try {
      const hasPermission = await this.ensurePermission();
      if (hasPermission) {
        await sendNotification({
          title,
          body: body || title,
        });
      }
    } catch (error) {
      console.error("Failed to show notification:", error);
    }
  }

  /**
   * Generic show method for compatibility
   */
  async show(
    title: string,
    message?: string,
    type: "error" | "success" | "info" | "warning" = "info"
  ) {
    switch (type) {
      case "error":
        await this.showError(title, message);
        break;
      case "success":
        await this.showSuccess(title, message);
        break;
      case "warning":
        await this.showWarning(title, message);
        break;
      default:
        await this.showInfo(title, message);
    }
  }
}

export const notificationHelper = new NotificationHelper();
