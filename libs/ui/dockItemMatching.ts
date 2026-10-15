/**
 * Identity rules for dock items and app windows, shared by the dock and the
 * app-picker so both agree on what counts as "the same app" when deduplicating.
 *
 * A dock item can point to the same program through several addresses: a `.lnk`
 * shortcut (stored in `path`), the resolved target executable (stored in
 * `relaunch.command`), or the executable itself. An app window reports the
 * executable path of its process plus an optional AppUserModelId. Two records
 * are the same app if any of those addresses coincide.
 *
 * On update, keep src/background/widgets/dock/cli.rs `get_windows_for_item` in
 * sync (it mirrors the window-matching half of this file).
 */

/** Path strings compare case-insensitively, as the Windows file system does. */
function samePath(a: string | null | undefined, b: string | null | undefined): boolean {
  if (!a || !b) return false;
  const aLower = a.toLowerCase();
  if (aLower === "") return false;
  return aLower === b.toLowerCase();
}

/** Identity fields shared by dock items and app windows. */
export interface AppIdentity {
  umid: string | null;
  path: string;
  relaunchCommand?: string | null;
}

/**
 * Whether two records refer to the same application.
 *
 * Matching umids win immediately; otherwise any exe-path coincidence decides:
 * item path vs window process path, or relaunch command vs process path.
 * Note the asymmetry with the pre-fix behavior: a window umid that differs from
 * the item's no longer hides an exe-path match — apps whose shortcut umid
 * doesn't survive into the window (or that have no shortcut umid at all) used
 * to duplicate on launch because of it.
 */
export function isSameApp(a: AppIdentity, b: AppIdentity): boolean {
  if (a.umid && b.umid && a.umid === b.umid) {
    return true;
  }
  return (
    samePath(a.path, b.path) ||
    samePath(a.relaunchCommand, b.path) ||
    samePath(a.path, b.relaunchCommand) ||
    samePath(a.relaunchCommand, b.relaunchCommand)
  );
}
