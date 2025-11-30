import { useEffect } from "react";
import { useDispatch } from "react-redux";
import { tauriEvents, debugtronAPI } from "../api/tauri";
import {
  appsFound,
  sessionAdded,
  sessionRemoved,
  sessionLogAppended,
  sessionPageUpdated,
  targetRegistered,
  targetUnregistered,
  type AppDispatch,
  type PageInfo,
} from "../store";

export function useTauriEvents() {
  const dispatch = useDispatch<AppDispatch>();

  useEffect(() => {
    const unlisteners: Array<() => void> = [];

    // Setup event listeners
    const setup = async () => {
      console.log('[TAURI EVENTS] Setting up event listeners...');

      // Apps updated
      const unlistenApps = await tauriEvents.onAppsUpdated((apps) => {
        console.log('[TAURI EVENTS] Received apps-updated event:', apps.length, 'apps');
        dispatch(appsFound(apps));
      });
      unlisteners.push(unlistenApps);
      console.log('[TAURI EVENTS] apps-updated listener registered');

      // Session added
      const unlistenSessionAdded = await tauriEvents.onSessionAdded((session) => {
        console.log('[TAURI EVENTS] Received session-added event:', session);
        dispatch(
          sessionAdded({
            sessionId: session.connectionId, // Use connectionId as sessionId (matches backend)
            appId: session.appId,
            targetId: session.targetId,
            connection: session.connection,
          })
        );
      });
      unlisteners.push(unlistenSessionAdded);
      console.log('[TAURI EVENTS] session-added listener registered');

      // Session removed
      const unlistenSessionRemoved = await tauriEvents.onSessionRemoved((sessionId) => {
        dispatch(sessionRemoved(sessionId));
      });
      unlisteners.push(unlistenSessionRemoved);

      // Session log
      const unlistenSessionLog = await tauriEvents.onSessionLog((log) => {
        const content = `[${log.type.toUpperCase()}] ${log.message}\n`;
        dispatch(sessionLogAppended({ sessionId: log.sessionId, content }));
      });
      unlisteners.push(unlistenSessionLog);

      // Pages updated
      const unlistenPages = await tauriEvents.onPagesUpdated((data) => {
        try {
          console.log('[TAURI EVENTS] Raw pages data:', data);
          const pages = JSON.parse(data) as Record<string, PageInfo[]>;
          console.log('[TAURI EVENTS] Parsed pages:', pages);
          console.log('[TAURI EVENTS] Pages keys:', Object.keys(pages));
          dispatch(sessionPageUpdated(pages));
        } catch (e) {
          console.error("Failed to parse pages data:", e, "Raw data:", data);
        }
      });
      unlisteners.push(unlistenPages);

      // Target registered
      const unlistenTargetReg = await tauriEvents.onTargetRegistered((target) => {
        console.log('[TAURI EVENTS] Received target-registered event:', target);
        dispatch(targetRegistered(target));
      });
      unlisteners.push(unlistenTargetReg);
      console.log('[TAURI EVENTS] target-registered listener registered');

      // Target unregistered
      const unlistenTargetUnreg = await tauriEvents.onTargetUnregistered((targetId) => {
        console.log('[TAURI EVENTS] Received target-unregistered event:', targetId);
        dispatch(targetUnregistered(targetId));
      });
      unlisteners.push(unlistenTargetUnreg);

      // Device removed
      const unlistenDeviceRemoved = await tauriEvents.onDeviceRemoved((targetId) => {
        console.log('[TAURI EVENTS] Received device-removed event:', targetId);
        dispatch(targetUnregistered(targetId));
      });
      unlisteners.push(unlistenDeviceRemoved);

      console.log('[TAURI EVENTS] All event listeners registered successfully');

      // Fetch initial data after listeners are registered
      console.log('[TAURI EVENTS] Fetching initial data...');

      try {
        const targets = await debugtronAPI.getTargets();
        console.log('[TAURI EVENTS] Fetched', targets.length, 'targets');
        targets.forEach(target => {
          dispatch(targetRegistered(target));
        });

        const apps = await debugtronAPI.getApps();
        console.log('[TAURI EVENTS] Fetched', apps.length, 'apps');
        dispatch(appsFound(apps));
      } catch (err) {
        console.error('[TAURI EVENTS] Failed to fetch initial data:', err);
      }
    };

    setup().catch(err => {
      console.error('[TAURI EVENTS] Failed to setup event listeners:', err);
    });

    // Cleanup
    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [dispatch]);
}
