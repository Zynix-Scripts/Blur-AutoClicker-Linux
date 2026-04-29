import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { useState } from "react";
import "./Updatebanner.css";

interface UpdateBannerProps {
  currentVersion: string;
  latestVersion: string;
}

type UpdateStage = "ready" | "installing" | "restart-required" | "error";

export default function UpdateBanner({
  currentVersion,
  latestVersion,
}: UpdateBannerProps) {
  const [stage, set_stage] = useState<UpdateStage>("ready");
  const [status_text, set_status_text] = useState<string | null>(null);

  const handle_update = async () => {
    try {
      set_stage("installing");
      set_status_text("Preparing update...");

      const update = await check();
      if (!update) {
        set_stage("ready");
        set_status_text("Update is no longer available.");
        return;
      }

      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            set_status_text("Downloading update...");
            break;
          case "Progress":
            set_status_text("Installing update...");
            break;
          case "Finished":
            set_status_text("Update installed. Restart to apply it.");
            break;
        }
      });

      set_stage("restart-required");
      set_status_text("Update installed. Restart to apply it.");
    } catch (err) {
      console.error("Failed to install update:", err);
      set_stage("error");
      set_status_text("Update install failed.");
    }
  };

  const handle_restart = async () => {
    try {
      await relaunch();
    } catch (err) {
      console.error("Failed to relaunch app:", err);
      set_stage("error");
      set_status_text("Restart failed. Please reopen the app manually.");
    }
  };

  return (
    <div className="update-banner">
      <span className="update-banner-text-old-version">v{currentVersion}</span>
      <span className="update-banner-text">to</span>

      <span className="update-banner-text-new-version">{latestVersion}</span>
      {status_text && (
        <span className="update-banner-status" data-stage={stage}>
          {status_text}
        </span>
      )}
      {stage === "restart-required" ? (
        <button className="update-banner-btn" onClick={handle_restart}>
          Restart to Apply Update
        </button>
      ) : (
        <button
          className="update-banner-btn"
          onClick={handle_update}
          disabled={stage === "installing"}
        >
          {stage === "installing" ? "Installing..." : "Download and Install"}
        </button>
      )}
    </div>
  );
}
