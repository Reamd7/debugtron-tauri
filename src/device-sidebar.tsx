import { type FC, useState } from "react";
import { useSelector } from "react-redux";
import type { RootState } from "./store";
import { cn } from "./lib/utils";

interface DeviceSidebarProps {
  selectedDeviceId?: string;
  onDeviceSelect: (deviceId: string) => void;
  onAddDevice: () => void;
  onDeviceDetails: (deviceId: string) => void;
}

export const DeviceSidebar: FC<DeviceSidebarProps> = ({
  selectedDeviceId,
  onDeviceSelect,
  onAddDevice,
  onDeviceDetails,
}) => {
  const targetStore = useSelector((state: RootState) => state.target);
  const [hoveredDevice, setHoveredDevice] = useState<string>();

  const devices = Object.values(targetStore).sort((a, b) => {
    // Local devices first
    if (a.type === "local" && b.type !== "local") return -1;
    if (a.type !== "local" && b.type === "local") return 1;
    return a.name.localeCompare(b.name);
  });

  return (
    <div className="w-[200px] bg-background border-r border-border flex flex-col h-full">
      <div className="px-4 py-2.5 font-bold text-xs text-muted-foreground uppercase tracking-wide">
        Devices
      </div>

      <div className="flex-1 overflow-y-auto">
        {devices.map((device) => {
          const isSelected = device.id === selectedDeviceId;
          const isHovered = device.id === hoveredDevice;
          const deviceIcon = device.type === "local" ? "💻" : "🌐";

          return (
            <div
              key={device.id}
              className={cn(
                "px-4 py-2.5 cursor-pointer transition-colors flex items-center justify-between",
                isSelected
                  ? "bg-primary text-primary-foreground"
                  : isHovered
                    ? "bg-accent"
                    : "transparent",
              )}
              onClick={() => {
                onDeviceSelect(device.id);
              }}
              onMouseEnter={() => {
                setHoveredDevice(device.id);
              }}
              onMouseLeave={() => {
                setHoveredDevice(undefined);
              }}
            >
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-base">{deviceIcon}</span>
                  <span className="text-sm font-medium truncate">
                    {device.name}
                  </span>
                </div>
                <div className="flex items-center gap-1.5 mt-1 ml-6">
                  <span
                    className={cn(
                      "w-2 h-2 rounded-full inline-block",
                      device.status === "connected" ? "bg-green-500" : "bg-gray-500",
                    )}
                  />
                  <span
                    className={cn(
                      "text-[11px]",
                      isSelected ? "text-primary-foreground/80" : "text-muted-foreground",
                    )}
                  >
                    {device.status}
                  </span>
                </div>
              </div>

              {isHovered && (
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onDeviceDetails(device.id);
                  }}
                  className={cn(
                    "p-1.5 rounded hover:bg-accent/50 transition-colors",
                    isSelected && "hover:bg-primary-foreground/20",
                  )}
                >
                  <svg
                    width="16"
                    height="16"
                    viewBox="0 0 16 16"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg"
                    className="w-4 h-4"
                  >
                    <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="1" fill="none" />
                    <circle cx="8" cy="5.5" r="0.5" fill="currentColor" />
                    <path d="M8 7v4" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
                  </svg>
                </button>
              )}
            </div>
          );
        })}
      </div>

      <div
        className="p-2.5 border-t border-border"
        style={{
          // @ts-expect-error - Non-standard property
          WebkitAppRegion: "no-drag",
        }}
      >
        <button
          onClick={onAddDevice}
          className="w-full px-4 py-2 text-sm font-medium text-foreground bg-secondary hover:bg-secondary/80 rounded-md transition-colors flex items-center justify-center gap-2"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M8 3.5v9M3.5 8h9" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
          </svg>
          Add Device
        </button>
      </div>
    </div>
  );
};
