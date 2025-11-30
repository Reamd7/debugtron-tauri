import * as Tabs from "@radix-ui/react-tabs";
import { type FC, useEffect, useMemo, useState } from "react";
import { useSelector } from "react-redux";
import { invoke } from "@tauri-apps/api/tauri";
import type { RootState } from "./store";
import { cn } from "./lib/utils";

import { Xterm } from "./xterm";

export const Session: FC = () => {
  const [activeId, setActiveId] = useState("");
  const [copiedUrl, setCopiedUrl] = useState<string | null>(null);
  const appStore = useSelector((state: RootState) => state.app);
  const sessionStore = useSelector((state: RootState) => state.session);
  const targetStore = useSelector((state: RootState) => state.target);

  // Copy to clipboard handler
  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedUrl(text);
      setTimeout(() => setCopiedUrl(null), 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  };

  // Memoize xterm options to prevent recreation on every render
  const xtermOptions = useMemo(() => ({
    fontFamily: "SFMono-Regular, Consolas, Liberation Mono, Menlo, monospace",
    convertEol: true,
  }), []);

  useEffect(() => {
    const sessionIds = Object.keys(sessionStore);

    // Ensure there always be one tab active
    if (!sessionIds.includes(activeId) && sessionIds[0]) {
      setActiveId(sessionIds[0]);
    }
  }, [activeId, sessionStore]);

  return (
    <Tabs.Root value={activeId} onValueChange={setActiveId} className="flex flex-col h-full">
      <Tabs.List className="flex border-b border-border bg-background px-2 gap-1 flex-shrink-0">
        {Object.entries(sessionStore).map(([id, session]) => {
          const appInfo = appStore[session.appId];
          const targetInfo = targetStore[session.targetId];

          const appPath = appInfo?.exePath ? ` - ${appInfo.exePath}` : "";
          const tabTitle = appInfo
            ? `${appInfo.name} (${targetInfo?.name ?? "Unknown"})${appPath}`
            : "Unnamed App";

          return (
            <Tabs.Trigger
              key={id}
              value={id}
              className={cn(
                "px-4 py-2 text-sm font-medium transition-colors border-b-2 border-transparent",
                "data-[state=active]:border-primary data-[state=active]:text-foreground",
                "data-[state=inactive]:text-muted-foreground hover:text-foreground",
              )}
            >
              {tabTitle}
            </Tabs.Trigger>
          );
        })}
      </Tabs.List>

      {Object.entries(sessionStore).map(([id, session]) => {
        const targetInfo = targetStore[session.targetId];

        return (
          <Tabs.Content
            key={id}
            value={id}
            className="flex-1 flex flex-col overflow-hidden"
          >
            {/* Connection Info */}
            {targetInfo && (
              <div
                className={cn(
                  "mx-4 mt-2.5 mb-2 p-3 rounded border",
                  session.connection.type === "local-process"
                    ? "bg-muted border-border"
                    : "bg-primary/10 border-primary/30",
                )}
              >
                <div className="font-semibold text-sm">
                  Target: {targetInfo.name} ({session.connection.type})
                </div>
                {session.connection.websocketUrl && (
                  <div className="mt-1 text-xs text-muted-foreground font-mono">
                    WebSocket: {session.connection.websocketUrl}
                  </div>
                )}
                {session.connection.nodePort && (
                  <div className="mt-1 text-xs text-muted-foreground">
                    Node Port: {session.connection.nodePort} | Renderer Port: {session.connection.windowPort}
                  </div>
                )}
              </div>
            )}

            {/* Top: Process Table */}
            <div className="h-[40%] min-h-[200px] overflow-auto px-4 border-b border-border flex-shrink-0">
              <table className="w-full text-sm">
                <thead className="sticky top-0 bg-background border-b border-border">
                  <tr>
                    <th className="text-left py-2 font-medium">Type</th>
                    <th className="text-left py-2 font-medium">Title</th>
                    <th className="text-left py-2 font-medium">DevTools Links</th>
                  </tr>
                </thead>
                <tbody>
                  {Object.entries(session.page).map(([pageId, page]) => {
                    // Extract WebSocket URL from devtoolsFrontendUrl
                    const wsMatch = page.devtoolsFrontendUrl.match(/ws=([^&]+)/);
                    const wsUrl = wsMatch ? wsMatch[1] : null;

                    // Generate devtools:// URLs based on page type
                    // page type (renderer with DOM) -> use inspector.html
                    // node type (main process, no DOM) -> use js_app.html
                    let devtoolsUrl: string | null = null;
                    if (wsUrl) {
                      if (page.type === "page") {
                        devtoolsUrl = `devtools://devtools/bundled/inspector.html?ws=${wsUrl}`;
                      } else if (page.type === "node") {
                        devtoolsUrl = `devtools://devtools/bundled/js_app.html?ws=${wsUrl}`;
                      } else {
                        // Default to inspector.html for other types
                        devtoolsUrl = `devtools://devtools/bundled/inspector.html?ws=${wsUrl}`;
                      }
                    }

                    return (
                      <tr key={pageId} className="border-b border-border hover:bg-accent/50">
                        <td className="py-2">
                          <span
                            className={cn(
                              "inline-block px-2 py-0.5 rounded text-xs font-medium",
                              page.type === "node"
                                ? "bg-green-500/10 text-green-600 dark:text-green-400"
                                : page.type === "page"
                                  ? "bg-blue-500/10 text-blue-600 dark:text-blue-400"
                                  : "bg-gray-500/10 text-gray-600 dark:text-gray-400",
                            )}
                          >
                            {page.type}
                          </span>
                        </td>
                        <td className="py-2 max-w-[300px] truncate">
                          {page.title}
                        </td>
                        <td className="py-2">
                          {devtoolsUrl ? (
                            <div className="flex items-center gap-2">
                              <button
                                onClick={async () => {
                                  console.log("[FRONTEND] Inspect button clicked, URL:", devtoolsUrl);
                                  try {
                                    console.log("[FRONTEND] Invoking open_devtools_window command...");
                                    await invoke("open_devtools_window", { url: devtoolsUrl });
                                    console.log("[FRONTEND] open_devtools_window command succeeded");
                                  } catch (error) {
                                    console.error("[FRONTEND] Failed to open DevTools window:", error);
                                    alert(`Failed to open DevTools window: ${error}`);
                                  }
                                }}
                                className="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded font-medium transition-colors"
                              >
                                Inspect
                              </button>
                              <button
                                onClick={async () => {
                                  try {
                                    await invoke("open_devtools", { url: devtoolsUrl });
                                  } catch (error) {
                                    console.error("Failed to open DevTools:", error);
                                  }
                                }}
                                className="px-3 py-1.5 bg-green-600 hover:bg-green-700 text-white text-sm rounded font-medium transition-colors"
                              >
                                Open in Browser
                              </button>
                              <button
                                onClick={(e) => {
                                  e.preventDefault();
                                  copyToClipboard(devtoolsUrl);
                                }}
                                className="text-xs text-blue-600 dark:text-blue-400 hover:underline"
                                title="Copy DevTools URL"
                              >
                                Copy URL
                              </button>
                              {copiedUrl === devtoolsUrl && (
                                <div className="px-2 py-1 bg-green-600 text-white text-xs rounded shadow-lg whitespace-nowrap">
                                  ✓ Copied
                                </div>
                              )}
                            </div>
                          ) : (
                            <span className="text-muted-foreground text-xs">No WebSocket URL</span>
                          )}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>

            {/* Bottom: Console Output */}
            <div className="flex-1 overflow-auto p-2.5 min-h-0">
              <Xterm
                content={session.log}
                options={xtermOptions}
              />
            </div>
          </Tabs.Content>
        );
      })}
    </Tabs.Root>
  );
};
