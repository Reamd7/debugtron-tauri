import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import type { AppInfo, TargetInfo } from "../store";

// Tauri command wrappers
export const debugtronAPI = {
  // Debug commands
  debug: async (appInfo: AppInfo, inspectBrk?: boolean): Promise<void> => {
    return invoke("debug", { appInfo, inspectBrk: inspectBrk || null });
  },

  debugPath: async (path: string, inspectBrk?: boolean): Promise<void> => {
    return invoke("debug_path", { path, inspectBrk: inspectBrk || null });
  },

  openDevtools: async (url: string): Promise<void> => {
    return invoke("open_devtools", { url });
  },

  // Device management
  addRemoteDevice: async (options: {
    host: string;
    port?: number;
    type: string;
  }): Promise<void> => {
    return invoke("add_remote_device", { options });
  },

  removeDevice: async (targetId: string): Promise<void> => {
    return invoke("remove_device", { targetId });
  },

  refreshDeviceApps: async (targetId: string): Promise<void> => {
    return invoke("refresh_device_apps", { targetId });
  },

  // Data retrieval
  getTargets: async (): Promise<TargetInfo[]> => {
    return invoke("get_targets");
  },

  getApps: async (): Promise<AppInfo[]> => {
    return invoke("get_apps");
  },
};

// Event listeners
export interface SessionLogData {
  sessionId: string;
  connectionId: string;
  type: "stdout" | "stderr";
  message: string;
}

// DebugConnection from backend (matches Rust struct)
export interface DebugConnection {
  connectionId: string;
  appId: string;
  targetId: string;
  debugPorts: {
    node?: number;
    renderer?: number;
    websocket?: string;
  };
  processHandle?: number;
  connection: {
    type: "local-process" | "remote-adb" | "remote-websocket";
    nodePort?: number;
    windowPort?: number;
    websocketUrl?: string;
    debugUrls?: string[];
  };
}

export interface TauriEventListeners {
  onAppsUpdated: (callback: (apps: AppInfo[]) => void) => Promise<() => void>;
  onSessionAdded: (callback: (session: DebugConnection) => void) => Promise<() => void>;
  onSessionRemoved: (callback: (sessionId: string) => void) => Promise<() => void>;
  onSessionLog: (callback: (log: SessionLogData) => void) => Promise<() => void>;
  onPagesUpdated: (callback: (data: string) => void) => Promise<() => void>;
  onTargetRegistered: (callback: (target: TargetInfo) => void) => Promise<() => void>;
  onTargetUnregistered: (callback: (targetId: string) => void) => Promise<() => void>;
  onDeviceRemoved: (callback: (targetId: string) => void) => Promise<() => void>;
}

export const tauriEvents: TauriEventListeners = {
  onAppsUpdated: async (callback) => {
    return listen<AppInfo[]>("apps-updated", (event) => {
      callback(event.payload);
    });
  },

  onSessionAdded: async (callback) => {
    return listen<DebugConnection>("session-added", (event) => {
      callback(event.payload);
    });
  },

  onSessionRemoved: async (callback) => {
    return listen<string>("session-removed", (event) => {
      callback(event.payload);
    });
  },

  onSessionLog: async (callback) => {
    return listen<SessionLogData>("session-log", (event) => {
      callback(event.payload);
    });
  },

  onPagesUpdated: async (callback) => {
    return listen<string>("pages-updated", (event) => {
      callback(event.payload);
    });
  },

  onTargetRegistered: async (callback) => {
    return listen<TargetInfo>("target-registered", (event) => {
      callback(event.payload);
    });
  },

  onTargetUnregistered: async (callback) => {
    return listen<string>("target-unregistered", (event) => {
      callback(event.payload);
    });
  },

  onDeviceRemoved: async (callback) => {
    return listen<string>("device-removed", (event) => {
      callback(event.payload);
    });
  },
};
