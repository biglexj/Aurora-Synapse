import React, { useEffect } from "react";

interface ToastProps {
  message: string;
  type?: "info" | "success" | "warning";
  durationMs?: number;
  onClose: () => void;
}

export const Toast: React.FC<ToastProps> = ({
  message,
  type = "success",
  durationMs = 4000,
  onClose,
}) => {
  useEffect(() => {
    const timer = setTimeout(() => {
      onClose();
    }, durationMs);

    return () => clearTimeout(timer);
  }, [durationMs, onClose]);

  return (
    <div className={`floating-toast ${type}`} role="status" aria-live="polite">
      <span className="toast-icon">
        {type === "success" && "✅"}
        {type === "info" && "ℹ️"}
        {type === "warning" && "⚠️"}
      </span>
      <span className="toast-message">{message}</span>
    </div>
  );
};
