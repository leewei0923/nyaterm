import { useCallback, useEffect, useRef } from "react";

interface ResizeHandleProps {
  direction: "horizontal" | "vertical";
  onResize: (delta: number) => void;
  className?: string;
}

/** Draggable handle for horizontal or vertical resize. Calls onResize(delta) on drag. */
export default function ResizeHandle({ direction, onResize, className = "" }: ResizeHandleProps) {
  const startPos = useRef(0);
  const onResizeRef = useRef(onResize);

  // Keep the ref up to date
  useEffect(() => {
    onResizeRef.current = onResize;
  }, [onResize]);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      startPos.current = direction === "horizontal" ? e.clientX : e.clientY;

      const handleMouseMove = (ev: MouseEvent) => {
        const current = direction === "horizontal" ? ev.clientX : ev.clientY;
        const delta = current - startPos.current;
        startPos.current = current;
        onResizeRef.current(delta);
      };

      const handleMouseUp = () => {
        document.removeEventListener("mousemove", handleMouseMove);
        document.removeEventListener("mouseup", handleMouseUp);
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
      };

      document.body.style.cursor = direction === "horizontal" ? "col-resize" : "row-resize";
      document.body.style.userSelect = "none";
      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
    },
    [direction],
  );

  const isHorizontal = direction === "horizontal";

  return (
    <div
      className={`
        ${
          isHorizontal
            ? "w-[3px] shrink-0 cursor-col-resize hover:opacity-70 active:opacity-90 transition-opacity"
            : "h-[3px] shrink-0 cursor-row-resize hover:opacity-70 active:opacity-90 transition-opacity"
        } ${className}
      `}
      style={{ backgroundColor: "var(--df-border)" }}
      onMouseDown={handleMouseDown}
    />
  );
}
