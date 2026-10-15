import { DockBarCommand } from "@dockbar/lib";
import { invoke } from "@tauri-apps/api/core";

export class startup {
  static async enable(): Promise<void> {
    await invoke(DockBarCommand.SetAutoStart, { enabled: true });
  }

  static async disable(): Promise<void> {
    await invoke(DockBarCommand.SetAutoStart, { enabled: false });
  }

  static async isEnabled(): Promise<boolean> {
    return await invoke<boolean>("get_auto_start_status");
  }
}
