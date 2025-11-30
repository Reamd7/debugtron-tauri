import { createSlice, type PayloadAction } from "@reduxjs/toolkit";

export interface TargetInfo {
  id: string;
  type: "local" | "remote";
  name: string;
  status: "connected" | "disconnected" | "connecting";
  lastDiscovery?: number;
}

export const targetSlice = createSlice({
  name: "target",
  initialState: {} as Record<string, TargetInfo>,
  reducers: {
    registered: (state, { payload }: PayloadAction<TargetInfo>) => {
      state[payload.id] = payload;
    },
    unregistered: (state, { payload: targetId }: PayloadAction<string>) => {
      // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
      delete state[targetId];
    },
    statusUpdated: (
      state,
      {
        payload,
      }: PayloadAction<{ targetId: string; status: TargetInfo["status"] }>,
    ) => {
      const target = state[payload.targetId];
      if (target) {
        target.status = payload.status;
      }
    },
    discoveryCompleted: (
      state,
      { payload: targetId }: PayloadAction<string>,
    ) => {
      const target = state[targetId];
      if (target) {
        target.lastDiscovery = Date.now();
      }
    },
  },
});
