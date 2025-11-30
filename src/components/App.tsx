import React from "react";

export const App: React.FC = () => {
  return (
    <div className="h-screen flex flex-col bg-background">
      {/* Draggable Title Bar */}
      <div
        data-tauri-drag-region
        className="h-10 flex items-center justify-center border-b border-border bg-background text-sm font-medium text-foreground"
      >
        Debugtron
      </div>

      {/* Main Content Area */}
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        <div className="text-center">
          <h1 className="text-2xl font-bold mb-4">Debugtron Tauri</h1>
          <p>Project initialized successfully!</p>
          <p className="text-sm mt-2">Start implementing features...</p>
        </div>
      </div>
    </div>
  );
};
