import * as Dialog from "@radix-ui/react-dialog";
import { type FC } from "react";
import { useSelector } from "react-redux";
import type { RootState } from "./store";
import { cn } from "./lib/utils";

interface DeviceDetailsDialogProps {
  isOpen: boolean;
  deviceId?: string;
  onClose: () => void;
  onRemove?: (deviceId: string) => void;
  onRefresh?: (deviceId: string) => void;
}

export const DeviceDetailsDialog: FC<DeviceDetailsDialogProps> = ({
  isOpen,
  deviceId,
  onClose,
  onRemove,
  onRefresh,
}) => {
  const targetStore = useSelector((state: RootState) => state.target);
  const appStore = useSelector((state: RootState) => state.app);

  const device = deviceId ? targetStore[deviceId] : undefined;

  if (!device) {
    return null;
  }

  const deviceApps = Object.values(appStore).filter(
    (app) => app.targetId === deviceId,
  );

  const isLocal = device.type === "local";

  // Parse device info for remote devices
  let connectionInfo: { address?: string; port?: string } = {};
  if (!isLocal && device.id.startsWith("remote-adb-")) {
    const parts = device.id.replace("remote-adb-", "").split(":");
    connectionInfo = {
      address: parts[0],
      port: parts[1],
    };
  }

  return (
    <Dialog.Root
      open={isOpen}
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0" />
        <Dialog.Content className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 bg-background border border-border rounded-lg shadow-lg w-[500px] max-w-[90vw] max-h-[85vh] overflow-auto">
          <Dialog.Title className="text-lg font-semibold px-6 pt-6 pb-4 border-b border-border">
            Device Details
          </Dialog.Title>

          <div className="px-6 py-4 space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium text-muted-foreground">Device Name</label>
              <div className="flex items-center gap-2 px-3 py-2 bg-accent rounded">
                <span className="text-xl">
                  {device.type === "local" ? "💻" : "🌐"}
                </span>
                <span className="text-sm font-medium">
                  {device.name}
                </span>
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium text-muted-foreground">Type</label>
              <span
                className={cn(
                  "inline-block px-3 py-1.5 rounded text-sm font-medium",
                  device.type === "local"
                    ? "bg-green-500/10 text-green-600 dark:text-green-400"
                    : "bg-blue-500/10 text-blue-600 dark:text-blue-400",
                )}
              >
                {device.type === "local" ? "Local Device" : "Remote Device"}
              </span>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium text-muted-foreground">Status</label>
              <div className="flex items-center gap-2">
                <span
                  className={cn(
                    "w-2.5 h-2.5 rounded-full inline-block",
                    device.status === "connected" ? "bg-green-500" : "bg-gray-500",
                  )}
                />
                <span className="text-sm capitalize">
                  {device.status}
                </span>
              </div>
            </div>

            {!isLocal && connectionInfo.address && (
              <div className="space-y-2">
                <label className="text-sm font-medium text-muted-foreground">Connection</label>
                <div className="px-3 py-2 bg-accent rounded font-mono text-xs">
                  {connectionInfo.address}
                  {connectionInfo.port && `:${connectionInfo.port}`}
                </div>
              </div>
            )}

            <div className="space-y-2">
              <label className="text-sm font-medium text-muted-foreground">Apps Discovered</label>
              <div className="text-2xl font-bold text-primary">
                {deviceApps.length}
              </div>
            </div>

            {device.lastDiscovery && (
              <div className="space-y-2">
                <label className="text-sm font-medium text-muted-foreground">Last Discovery</label>
                <div className="text-sm text-muted-foreground">
                  {new Date(device.lastDiscovery).toLocaleString()}
                </div>
              </div>
            )}
          </div>

          <div className="px-6 py-4 border-t border-border flex justify-end gap-2">
            {!isLocal && onRemove && (
              <button
                onClick={() => {
                  onRemove(device.id);
                  onClose();
                }}
                className="px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-md transition-colors"
              >
                Remove Device
              </button>
            )}
            {onRefresh && (
              <button
                onClick={() => {
                  onRefresh(device.id);
                }}
                className="px-4 py-2 text-sm font-medium text-foreground bg-secondary hover:bg-secondary/80 rounded-md transition-colors flex items-center gap-2"
              >
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <path
                    d="M12 7A5 5 0 1 1 7 2M7 2V5M7 2L5 4"
                    stroke="currentColor"
                    strokeWidth="1.2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
                Refresh Apps
              </button>
            )}
            <button
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-foreground bg-secondary hover:bg-secondary/80 rounded-md transition-colors"
            >
              Close
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};
