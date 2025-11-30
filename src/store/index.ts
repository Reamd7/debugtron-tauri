import { configureStore } from "@reduxjs/toolkit";
import { appSlice } from "./app";
import { sessionSlice } from "./session";
import { targetSlice } from "./target";

export const store = configureStore({
  reducer: {
    app: appSlice.reducer,
    session: sessionSlice.reducer,
    target: targetSlice.reducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;

// Export actions
export const { found: appsFound } = appSlice.actions;
export const {
  added: sessionAdded,
  removed: sessionRemoved,
  pageUpdated: sessionPageUpdated,
  logAppended: sessionLogAppended,
} = sessionSlice.actions;
export const {
  registered: targetRegistered,
  unregistered: targetUnregistered,
  statusUpdated: targetStatusUpdated,
  discoveryCompleted: targetDiscoveryCompleted,
} = targetSlice.actions;

// Export types
export type { AppInfo } from "./app";
export type { SessionInfo, PageInfo } from "./session";
export type { TargetInfo } from "./target";
