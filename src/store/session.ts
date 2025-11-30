import { createSlice, type PayloadAction } from "@reduxjs/toolkit";

export interface PageInfo {
  description: string;
  devtoolsFrontendUrl: string;
  id: string;
  title: string;
  type: "node" | "page" | "webview";
  url: string;
}

export interface SessionInfo {
  appId: string;
  targetId: string; // New: which target this session belongs to
  page: Record<string, PageInfo>;
  log: string;

  // Connection details (flexible for different target types)
  connection: {
    type: "local-process" | "remote-adb" | "remote-websocket";
    nodePort?: number;
    windowPort?: number;
    websocketUrl?: string;
    debugUrls?: string[];
  };

  // Legacy fields for backward compatibility (deprecated)
  nodePort?: number;
  windowPort?: number;
}

export const sessionSlice = createSlice({
  name: "session",
  initialState: {} as Record<string, SessionInfo>,
  reducers: {
    added: (
      state,
      {
        payload: { sessionId, ...rest },
      }: PayloadAction<{
        sessionId: string;
        appId: string;
        targetId: string;
        connection: {
          type: "local-process" | "remote-adb" | "remote-websocket";
          nodePort?: number;
          windowPort?: number;
          websocketUrl?: string;
          debugUrls?: string[];
        };
      }>,
    ) => {
      state[sessionId] = {
        ...rest,
        page: {},
        log: "",
        // Legacy fields for backward compatibility
        nodePort: rest.connection.nodePort,
        windowPort: rest.connection.windowPort,
      };
    },
    pageUpdated: (state, { payload }: PayloadAction<Record<string, PageInfo[]>>) => {
      // Payload is now a map of sessionId -> pages array
      console.log('[SESSION REDUCER] pageUpdated called with payload:', payload);
      console.log('[SESSION REDUCER] Current state sessions:', Object.keys(state));
      Object.entries(payload).forEach(([sessionId, pages]) => {
        console.log('[SESSION REDUCER] Processing sessionId:', sessionId, 'with', pages.length, 'pages');
        const session = state[sessionId];
        if (session && pages) {
          console.log('[SESSION REDUCER] Updating session:', sessionId);
          session.page = {};
          pages.sort((a, b) => (a.id < b.id ? -1 : 1)).forEach((p) => {
            session.page[p.id] = p;
          });
        } else {
          console.warn('[SESSION REDUCER] Session not found or no pages:', sessionId, 'session exists:', !!session);
        }
      });
    },
    logAppended: (
      state,
      {
        payload: { sessionId, content },
      }: PayloadAction<{ sessionId: string; content: string }>,
    ) => {
      const selected = state[sessionId];
      if (selected) selected.log += content;
    },
    removed: (state, { payload: sessionId }: PayloadAction<string>) => {
      // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
      delete state[sessionId];
    },
  },
});
