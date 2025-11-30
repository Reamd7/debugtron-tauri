import { createSlice,
  type PayloadAction,

} from "@reduxjs/toolkit";

export interface AppInfo {
  id: string; // Now includes target prefix: 'local-macos:com.app' or 'remote-adb-192.168.1.100:com.app'
  name: string;
  icon: string;
  exePath?: string; // Optional, only for local

  // Target metadata
  targetId: string; // Reference to target adapter
  targetType: "local" | "remote";
  metadata?: {
    platform?: string; // 'win32' | 'darwin' | 'linux'
    deviceInfo?: string; // For remote: 'Android TV 12'
    packageName?: string; // For Android
  };
}

export const appSlice = createSlice({
  name: "app",
  initialState: {} as Record<string, AppInfo>,
  reducers: {
    found: (state, { payload }: PayloadAction<AppInfo[]>) => {
      // Clear existing state
      Object.keys(state).forEach((key) => {
        // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
        delete state[key];
      });
      payload
        .sort((a, b) => (a.id < b.id ? -1 : 1))
        .forEach((appInfo) => {
          state[appInfo.id] = appInfo;
        });
    },
  },
});
