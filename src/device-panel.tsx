import { type FC, useState, useRef, useEffect } from "react";
import { useSelector } from "react-redux";
import type { RootState } from "./store";
import { debugtronAPI } from "./api/tauri";
import type { AppInfo } from "./store";
import { listen } from "@tauri-apps/api/event";

import defaultImage from "./images/electron.png";

interface DevicePanelProps {
  selectedDeviceId?: string;
  onDeviceSettings: () => void;
}

export const DevicePanel: FC<DevicePanelProps> = ({
  selectedDeviceId,
  onDeviceSettings,
}) => {
  const appStore = useSelector((state: RootState) => state.app);
  const targetStore = useSelector((state: RootState) => state.target);
  const [input, setInput] = useState("");
  const contextMenuRef = useRef<HTMLDivElement>(null);
  const device = selectedDeviceId ? targetStore[selectedDeviceId] : undefined;

  // Context menu state that handles either app or path
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; app?: AppInfo; path?: string } | null>(null);
  // Ref to track if we've already adjusted the menu position
  const menuPositionAdjustedRef = useRef(false);

  // Close context menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (contextMenuRef.current && !contextMenuRef.current.contains(event.target as Node)) {
        setContextMenu(null);
      }
    };

    if (contextMenu) {
      document.addEventListener("mousedown", handleClickOutside);
      // Reset position flag when context menu opens
      menuPositionAdjustedRef.current = false;
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [contextMenu]);

  // Remove the useEffect that causes flickering

  // Listen for Tauri file drop events
  useEffect(() => {
    const unlisten = listen('tauri://file-drop', (event: { payload: unknown }) => {
      console.log('Tauri file drop event received:', event);

      // Check what's in the payload
      if (event.payload) {
        console.log('Event payload:', event.payload);
        console.log('Event payload type:', Array.isArray(event.payload) ? 'array' : 'object');

        // Tauri is passing the paths array directly as payload!
        let paths: string[] = [];

        if (Array.isArray(event.payload)) {
          // Payload is directly an array of paths
          paths = event.payload as string[];
        } else if (typeof event.payload === 'object' && event.payload !== null) {
          // Check if it's an object with paths property
          const payloadObj = event.payload as Record<string, unknown>;
          if (Array.isArray(payloadObj.paths)) {
            paths = payloadObj.paths as string[];
          } else if (typeof payloadObj.path === 'string') {
            // Fallback: single path
            paths = [payloadObj.path];
          }
        }

        if (paths.length > 0) {
          const droppedPath = paths[0] as string;
          console.log('Setting input to:', droppedPath);
          setInput(droppedPath);
        } else {
          console.log('No paths found in payload');
        }
      } else {
        console.log('Event payload is undefined');
      }
    });

    // Cleanup the listener on component unmount
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  // Filter apps for the selected device
  const deviceApps = Object.values(appStore).filter(
    (app) => app.targetId === selectedDeviceId,
  );

  if (!device) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground text-base">
        Select a device to view apps
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Device Header */}
      <div
        className="px-4 py-2.5 border-b border-border bg-background flex items-center justify-between"
        style={{
          // @ts-expect-error - Non-standard property
          WebkitAppRegion: "no-drag",
        }}
      >
        <div className="flex items-center gap-3">
          <span className="text-xl">
            {device.type === "local" ? "💻" : "🌐"}
          </span>
          <div>
            <div className="text-base font-semibold">
              {device.name}
            </div>
            <div className="text-xs text-muted-foreground">
              {deviceApps.length} app{deviceApps.length !== 1 ? "s" : ""} found
            </div>
          </div>
        </div>
        <button
          onClick={onDeviceSettings}
          className="p-2 rounded hover:bg-accent transition-colors"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path
              d="M8 10a2 2 0 1 0 0-4 2 2 0 0 0 0 4z"
              stroke="currentColor"
              strokeWidth="1"
              fill="none"
            />
            <path
              d="M14 8a6 6 0 1 1-12 0 6 6 0 0 1 12 0z"
              stroke="currentColor"
              strokeWidth="1"
              fill="none"
            />
          </svg>
        </button>
      </div>

      {/* App List */}
      <div className="flex-1 overflow-y-auto p-4">
        {deviceApps.length === 0
          ? (
            <div className="text-center text-muted-foreground py-10 px-5">
              No apps found on this device
            </div>
          )
          : (
            <div className="flex flex-col gap-2">
              {deviceApps.map((app) => (
                <div
                  key={app.id}
                  className="p-3 border border-border rounded hover:bg-accent/50 transition-colors cursor-pointer flex items-center gap-3"
                  onClick={() => {
                    debugtronAPI.debug(app, false);
                  }}
                  onContextMenu={(e) => {
                    e.preventDefault();
                    setContextMenu({ x: e.clientX, y: e.clientY, app });
                  }}
                >
                  <img
                    src={app.icon || defaultImage}
                    alt=""
                    className="w-8 h-8 rounded"
                  />
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium truncate">
                      {app.name}
                    </div>
                    <div className="text-xs text-muted-foreground truncate">
                      {app.metadata?.packageName ?? app.id.split(":").pop()}
                    </div>
                  </div>
                  <button
                    className="p-2 rounded bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
                    title="Debug this app"
                  >
                    <svg
                      width="12"
                      height="12"
                      viewBox="0 0 12 12"
                      fill="none"
                      xmlns="http://www.w3.org/2000/svg"
                    >
                      <path d="M3 2l6 4-6 4V2z" fill="currentColor" />
                    </svg>
                  </button>
                </div>
              ))}
            </div>
          )}
      </div>

      {/* Context Menu */}
      {contextMenu && (
        <div
          ref={contextMenuRef}
          className="fixed bg-background border border-border rounded-md shadow-lg py-1 z-50"
          style={{
            left: `${contextMenu.x}px`,
            top: `${contextMenu.y}px`,
          }}
        >
          {/* Regular app debug */}
          {contextMenu.app && (
            <>
              <button
                className="w-full px-4 py-2 text-left text-sm hover:bg-accent transition-colors flex items-center gap-2"
                onClick={() => {
                  debugtronAPI.debug(contextMenu.app!, false);
                  setContextMenu(null);
                }}
              >
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <path d="M3 2l6 4-6 4V2z" fill="currentColor" />
                </svg>
                Debug
              </button>
              <button
                className="w-full px-4 py-2 text-left text-sm hover:bg-accent transition-colors flex items-center gap-2"
                onClick={() => {
                  debugtronAPI.debug(contextMenu.app!, true);
                  setContextMenu(null);
                }}
              >
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <circle cx="7" cy="7" r="2" fill="currentColor" />
                  <path d="M7 2v2M7 10v2M2 7h2M10 7h2" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
                </svg>
                Debug with --inspect-brk
              </button>
            </>
          )}

          {/* Custom path debug */}
          {contextMenu.path && (
            <>
              <button
                className="w-full px-4 py-2 text-left text-sm hover:bg-accent transition-colors flex items-center gap-2"
                onClick={() => {
                  debugtronAPI.debugPath(contextMenu.path!, false);
                  setContextMenu(null);
                }}
              >
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <path d="M3 2l6 4-6 4V2z" fill="currentColor" />
                </svg>
                Debug
              </button>
              <button
                className="w-full px-4 py-2 text-left text-sm hover:bg-accent transition-colors flex items-center gap-2"
                onClick={() => {
                  debugtronAPI.debugPath(contextMenu.path!, true);
                  setContextMenu(null);
                }}
              >
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <circle cx="7" cy="7" r="2" fill="currentColor" />
                  <path d="M7 2v2M7 10v2M2 7h2M10 7h2" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
                </svg>
                Debug with --inspect-brk
              </button>
            </>
          )}
        </div>
      )}

      {/* Custom Path Input (for local devices) */}
      {device.type === "local" && (
        <div
          className="p-4 border-t border-border bg-background"
          style={{
            // @ts-expect-error - Non-standard property
            WebkitAppRegion: "no-drag",
          }}
        >
          <div className="flex gap-2 flex-wrap">
            <input
              type="text"
              value={input}
              placeholder="App not found? Enter custom path..."
              onChange={(e) => {
                setInput(e.target.value);
              }}
              className="flex-1 px-3 py-2 bg-background border border-input rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-ring transition-colors min-w-[0]"
              title="Input custom path here and click Debug"
            />
            {/* Debug button with both left-click and right-click functionality */}
            <button
              onClick={() => {
                debugtronAPI.debugPath(input, false);
              }}
              onContextMenu={(e) => {
                e.preventDefault();
                if (input.trim()) {
                  // Calculate position before rendering - place menu above the button
                  // Estimate menu height (2 items * ~30px each + 4px border)
                  const estimatedMenuHeight = 64;
                  // Position menu above the mouse, prevent going off top of screen
                  const menuY = Math.max(0, e.clientY - estimatedMenuHeight);
                  setContextMenu({ x: e.clientX, y: menuY, path: input.trim() });
                }
              }}
              className="px-4 py-2 text-sm font-medium text-primary-foreground bg-primary hover:bg-primary/90 rounded-md transition-colors flex items-center gap-2"
              title="Left click: Debug, Right click: Debug with --inspect-brk"
            >
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M3 2l6 4-6 4V2z" fill="currentColor" />
              </svg>
              Debug
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
