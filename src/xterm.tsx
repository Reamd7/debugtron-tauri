import { CanvasAddon } from "@xterm/addon-canvas";
import { FitAddon } from "@xterm/addon-fit";
import { type ITerminalOptions, Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import { type FC, useEffect, useRef } from "react";

export const Xterm: FC<{ content: string; options?: ITerminalOptions }> = ({
  content,
  options,
}) => {
  const domRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const lastContentLengthRef = useRef<number>(0);

  useEffect(() => {
    const term = new Terminal(options);
    term.loadAddon(new FitAddon());
    term.loadAddon(new CanvasAddon());
    if (domRef.current) term.open(domRef.current);
    termRef.current = term;
    lastContentLengthRef.current = 0;

    return () => {
      termRef.current?.dispose();
    };
  }, [options]);

  useEffect(() => {
    if (!termRef.current) return;

    const lastLength = lastContentLengthRef.current;

    // If content is shorter than before, session was reset - clear and rewrite
    if (content.length < lastLength) {
      termRef.current.clear();
      termRef.current.write(content);
      lastContentLengthRef.current = content.length;
      return;
    }

    // Only append new content (incremental update)
    if (content.length > lastLength) {
      const newContent = content.slice(lastLength);
      termRef.current.write(newContent);
      lastContentLengthRef.current = content.length;
    }
  }, [content]);

  return <div ref={domRef} />;
};
