import React from "react";
import { UpdateCheckResult } from "./types";
import { sanitizeReleaseNotes } from "./updaterService";

interface UpdateModalDialogProps {
  update: UpdateCheckResult;
  isMobile: boolean;
  onClose: () => void;
}

export const UpdateModalDialog: React.FC<UpdateModalDialogProps> = ({
  update,
  isMobile,
  onClose,
}) => {
  const sanitizedNotes = sanitizeReleaseNotes(update.release_notes);

  return (
    <div className="update-modal-backdrop" onClick={onClose}>
      <div
        className="update-modal-dialog"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
      >
        <div className="update-modal-header">
          <div className="update-badge-icon">🚀</div>
          <div className="update-header-info">
            <span className="update-overtitle">NUEVA ACTUALIZACIÓN DISPONIBLE</span>
            <h2 className="update-title">{update.release_name}</h2>
            <div className="update-version-chips">
              <span className="version-chip current">Actual: v{update.current_version}</span>
              <span className="version-arrow">➔</span>
              <span className="version-chip next">Nueva: v{update.latest_version}</span>
            </div>
          </div>
          <button className="update-close-btn" onClick={onClose} aria-label="Cerrar">
            ✕
          </button>
        </div>

        <div className="update-notes-container">
          <span className="notes-label">Notas de la Versión:</span>
          <div className="update-notes-content">
            {sanitizedNotes.split("\n").map((line, idx) => {
              const trimmed = line.trim();
              if (!trimmed) return <div key={idx} className="note-spacer" />;
              return (
                <p key={idx} className="note-line">
                  {trimmed}
                </p>
              );
            })}
          </div>
        </div>

        <div className="update-modal-actions">
          {isMobile ? (
            update.apk_url ? (
              <a
                href={update.apk_url}
                target="_blank"
                rel="noreferrer"
                className="update-action-btn primary"
              >
                📱 Descargar APK Universal
              </a>
            ) : (
              <a
                href={update.html_url}
                target="_blank"
                rel="noreferrer"
                className="update-action-btn primary"
              >
                🌐 Ver en GitHub
              </a>
            )
          ) : (
            <>
              {update.exe_url && (
                <a
                  href={update.exe_url}
                  target="_blank"
                  rel="noreferrer"
                  className="update-action-btn primary"
                >
                  ⚡ Descargar Instalador (.exe)
                </a>
              )}
              {update.apk_url && (
                <a
                  href={update.apk_url}
                  target="_blank"
                  rel="noreferrer"
                  className="update-action-btn secondary"
                >
                  📱 Descargar APK
                </a>
              )}
              {!update.exe_url && !update.apk_url && (
                <a
                  href={update.html_url}
                  target="_blank"
                  rel="noreferrer"
                  className="update-action-btn primary"
                >
                  🌐 Abrir Release en GitHub
                </a>
              )}
            </>
          )}
          <button className="update-action-btn ghost" onClick={onClose}>
            Posponer
          </button>
        </div>
      </div>
    </div>
  );
};
