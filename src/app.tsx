import "./app.css";
import * as Dialog from "@radix-ui/react-dialog";
import * as RadioGroup from "@radix-ui/react-radio-group";
import React, { useEffect, useState } from "react";
import { useSelector } from "react-redux";
import type { RootState } from "./store";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { debugtronAPI } from "./api/tauri";

import { DeviceDetailsDialog } from "./device-details-dialog";
import { DevicePanel } from "./device-panel";
import { DeviceSidebar } from "./device-sidebar";
import { Session } from "./session";

const App: React.FC = () => {
  // Setup Tauri event listeners
  useTauriEvents();

  const sessionStore = useSelector((state: RootState) => state.session);
  const targetStore = useSelector((state: RootState) => state.target);

  // Device management state
  const [selectedDeviceId, setSelectedDeviceId] = useState<string>();
  const [deviceDetailsOpen, setDeviceDetailsOpen] = useState(false);
  const [deviceDetailsId, setDeviceDetailsId] = useState<string>();
  const [addDeviceOpen, setAddDeviceOpen] = useState(false);

  // Remote device dialog state
  const [deviceType, setDeviceType] = useState<"adb" | "ssh" | "websocket">("adb");
  const [deviceAddress, setDeviceAddress] = useState("");
  const [devicePort, setDevicePort] = useState("5555");

  // Auto-select first device (local) on startup
  useEffect(() => {
    if (!selectedDeviceId && Object.keys(targetStore).length > 0) {
      const devices = Object.values(targetStore);
      const localDevice = devices.find((d) => d.type === "local");
      if (localDevice) {
        setSelectedDeviceId(localDevice.id);
      } else if (devices[0]) {
        setSelectedDeviceId(devices[0].id);
      }
    }
  }, [targetStore, selectedDeviceId]);

  const handleAddDevice = async () => {
    try {
      await debugtronAPI.addRemoteDevice({
        host: deviceAddress,
        port: parseInt(devicePort, 10),
        type: deviceType,
      });
      setAddDeviceOpen(false);
      setDeviceAddress("");
      setDevicePort("5555");
    } catch (error) {
      console.error("Failed to add device:", error);
    }
  };

  return (
    <div className="h-screen flex flex-col bg-background">
      {/* Draggable Title Bar */}
      <div
        className="h-10 flex items-center justify-center border-b border-border bg-background text-sm font-medium text-foreground"
        style={{
          // @ts-expect-error - Non-standard property
          WebkitAppRegion: "drag",
        }}
      >
        Debugtron
      </div>

      {/* Main Content Area */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Sidebar - Device List */}
        <DeviceSidebar
          selectedDeviceId={selectedDeviceId}
          onDeviceSelect={setSelectedDeviceId}
          onAddDevice={() => {
            setAddDeviceOpen(true);
          }}
          onDeviceDetails={(deviceId) => {
            setDeviceDetailsId(deviceId);
            setDeviceDetailsOpen(true);
          }}
        />

        {/* Middle: App List - Always shows apps for selected device */}
        <div className="w-[300px] flex flex-col border-r border-border overflow-hidden">
          <DevicePanel
            selectedDeviceId={selectedDeviceId}
            onDeviceSettings={() => {
              if (selectedDeviceId) {
                setDeviceDetailsId(selectedDeviceId);
                setDeviceDetailsOpen(true);
              }
            }}
          />
        </div>

        {/* Right: Session Details - Shows debugging sessions */}
        {Object.keys(sessionStore).length > 0
          ? (
            <div className="flex-1 flex flex-col overflow-hidden">
              <Session />
            </div>
          )
          : (
            <div className="flex-1 flex items-center justify-center text-muted-foreground text-base">
              Select an app to start debugging
            </div>
          )}
      </div>

      {/* Device Details Dialog */}
      <DeviceDetailsDialog
        isOpen={deviceDetailsOpen}
        deviceId={deviceDetailsId}
        onClose={() => {
          setDeviceDetailsOpen(false);
        }}
        onRemove={(deviceId) => {
          debugtronAPI.removeDevice(deviceId);
          // Switch to local device if removing current device
          if (deviceId === selectedDeviceId) {
            const localDevice = Object.values(targetStore).find((d) => d.type === "local");
            if (localDevice) {
              setSelectedDeviceId(localDevice.id);
            }
          }
        }}
        onRefresh={(deviceId) => {
          debugtronAPI.refreshDeviceApps(deviceId);
        }}
      />

      {/* Add Remote Device Dialog */}
      <Dialog.Root open={addDeviceOpen} onOpenChange={setAddDeviceOpen}>
        <Dialog.Portal>
          <Dialog.Overlay className="fixed inset-0 bg-black/50 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0" />
          <Dialog.Content className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 bg-background border border-border rounded-lg shadow-lg w-[500px] max-w-[90vw] max-h-[85vh] overflow-auto">
            <Dialog.Title className="text-lg font-semibold px-6 pt-6 pb-4 border-b border-border">
              Add Remote Device
            </Dialog.Title>

            <div className="px-6 py-4 space-y-4">
              <div className="space-y-2">
                <label className="text-sm font-medium text-foreground">Device Type</label>
                <RadioGroup.Root
                  value={deviceType}
                  onValueChange={(value) => {
                    setDeviceType(value as "adb" | "ssh" | "websocket");
                  }}
                  className="space-y-2"
                >
                  <div className="flex items-center space-x-2">
                    <RadioGroup.Item
                      value="adb"
                      className="w-4 h-4 rounded-full border border-primary data-[state=checked]:bg-primary data-[state=checked]:border-primary"
                    >
                      <RadioGroup.Indicator className="flex items-center justify-center w-full h-full relative after:content-[''] after:block after:w-2 after:h-2 after:rounded-full after:bg-white" />
                    </RadioGroup.Item>
                    <label className="text-sm">ADB (Android Debug Bridge)</label>
                  </div>
                  <div className="flex items-center space-x-2 opacity-50">
                    <RadioGroup.Item
                      value="ssh"
                      disabled
                      className="w-4 h-4 rounded-full border border-muted"
                    >
                      <RadioGroup.Indicator />
                    </RadioGroup.Item>
                    <label className="text-sm">SSH (Coming soon)</label>
                  </div>
                  <div className="flex items-center space-x-2 opacity-50">
                    <RadioGroup.Item
                      value="websocket"
                      disabled
                      className="w-4 h-4 rounded-full border border-muted"
                    >
                      <RadioGroup.Indicator />
                    </RadioGroup.Item>
                    <label className="text-sm">WebSocket (Coming soon)</label>
                  </div>
                </RadioGroup.Root>
              </div>

              <div className="space-y-2">
                <label htmlFor="device-address" className="text-sm font-medium text-foreground">
                  Device Address
                </label>
                <input
                  id="device-address"
                  type="text"
                  placeholder="e.g., 192.168.1.100"
                  value={deviceAddress}
                  onChange={(e) => {
                    setDeviceAddress(e.target.value);
                  }}
                  className="w-full px-3 py-2 bg-background border border-input rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-ring"
                />
              </div>

              <div className="space-y-2">
                <label htmlFor="device-port" className="text-sm font-medium text-foreground">
                  Port
                </label>
                <input
                  id="device-port"
                  type="text"
                  placeholder="5555"
                  value={devicePort}
                  onChange={(e) => {
                    setDevicePort(e.target.value);
                  }}
                  className="w-full px-3 py-2 bg-background border border-input rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-ring"
                />
              </div>
            </div>

            <div className="px-6 py-4 border-t border-border flex justify-end gap-2">
              <button
                onClick={() => {
                  setAddDeviceOpen(false);
                }}
                className="px-4 py-2 text-sm font-medium text-foreground bg-secondary hover:bg-secondary/80 rounded-md transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleAddDevice}
                disabled={!deviceAddress}
                className="px-4 py-2 text-sm font-medium text-primary-foreground bg-primary hover:bg-primary/90 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Connect
              </button>
            </div>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </div>
  );
};

export default App;
