import type {
  DockBarCmdArgumentMap,
  Icon as IIcon,
  IconPack,
  IconPack as IIconPack,
  IconPackEntry,
} from "@dockbar/types";
import { List } from "../utils/List.ts";
import { newFromInvoke, newOnEvent } from "../utils/State.ts";
import { DockBarCommand, DockBarEvent, invoke, type UnSubscriber } from "../handlers/mod.ts";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";

type GetIconArgs = Extract<DockBarCmdArgumentMap, { [DockBarCommand.GetIcon]: unknown }>[DockBarCommand.GetIcon];

export class IconPackList extends List<IIconPack> {
  static getAsync(): Promise<IconPackList> {
    return newFromInvoke(this, DockBarCommand.StateGetIconPacks);
  }

  static onChange(cb: (payload: IconPackList) => void): Promise<UnSubscriber> {
    return newOnEvent(cb, this, DockBarEvent.StateIconPacksChanged);
  }
}

/**
 * Class helper to allow easy use of icon packs
 */
export class IconPackManager {
  private callbacks: Set<() => void> = new Set();
  private unlisteners: UnlistenFn[] = [];
  private isListeningForChanges = false;

  protected constructor(
    protected _availableIconPacks: IconPack[],
  ) {}

  get iconPacks(): IconPack[] {
    return this._availableIconPacks;
  }

  protected resolveAvailableIcons(): void {
    for (const pack of this._availableIconPacks) {
      const path = `${pack.metadata.path}`;

      if (pack.missing) {
        pack.missing = resolveIcon(path, pack.missing);
      }

      for (const entry of pack.entries) {
        if (entry.type === "unique") {
          entry.path = entry.path?.toLowerCase() || null;
        }

        if (entry.type === "shared") {
          entry.extension = entry.extension.toLowerCase();
        }

        if (entry.icon) {
          entry.icon = resolveIcon(path, entry.icon);
        }
      }
    }
  }

  /**
   * Creates an instance of IconPackManager. This intance will be updated when
   * the list of icon packs changes, so just having one global instance is enough.
   *
   * @returns A new instance of IconPackManager
   */
  public static async create(): Promise<IconPackManager> {
    const instance = new IconPackManager(
      (await IconPackList.getAsync()).asArray(),
    );
    instance.resolveAvailableIcons();
    return instance;
  }

  /**
   * Registers a callback to be invoked when the list of icon packs changes.
   *
   * @param {() => void} cb - The callback to be invoked when the list of icon packs changes.
   *                          This callback takes no arguments and returns no value.
   * @returns {Promise<UnlistenFn>} A promise that resolves to an `UnlistenFn` function. When invoked,
   *                                this function unregisters the callback and stops listening for changes.
   */
  public async onChange(cb: () => void): Promise<UnlistenFn> {
    this.callbacks.add(cb);

    if (!this.isListeningForChanges) {
      this.isListeningForChanges = true;
      const unlistenerIcons = await IconPackList.onChange((list) => {
        this._availableIconPacks = JSON.parse(JSON.stringify(list.all()));
        this.resolveAvailableIcons();
        this.callbacks.forEach((cb) => cb());
      });
      this.unlisteners = [unlistenerIcons];
    }

    return () => {
      this.callbacks.delete(cb);
      if (this.callbacks.size === 0) {
        this.unlisteners.forEach((unlisten) => unlisten());
        this.unlisteners = [];
        this.isListeningForChanges = false;
      }
    };
  }

  public getIconEntry(args: GetIconArgs): IconPackEntry | null {
    const {
      path,
      umid,
      __seen = new Set<string>(),
    } = args as GetIconArgs & { __seen?: Set<string> };

    // If neither path nor UMID is provided, return null
    if (!path && !umid) {
      return null;
    }

    const lowerPath = path?.toLowerCase();

    // Extract extension only when there's an actual dot after the last path separator,
    // to avoid treating directory names without dots as extensions.
    const lastDot = lowerPath?.lastIndexOf(".");
    const lastSlash = lowerPath?.lastIndexOf("\\") ?? -1;
    const extension = lastDot !== undefined && lastDot > lastSlash ? lowerPath!.slice(lastDot + 1) : undefined;

    // Add the starting path to __seen so that direct A→B→A cycles are caught in 2 hops.
    if (lowerPath) {
      __seen.add(lowerPath);
    }

    for (const pack of this._availableIconPacks.toReversed()) {
      let entry: Extract<IconPackEntry, { type: "unique" }> | undefined;

      if (umid) {
        entry = pack.entries.find(
          (e) => e.type === "unique" && !!e.umid && e.umid === umid,
        ) as Extract<IconPackEntry, { type: "unique" }>;
      }

      if (!entry && lowerPath) {
        entry = pack.entries.find((e) => {
          if (e.type !== "unique" || !e.path) {
            return false;
          }

          if (e.path === lowerPath) {
            return true;
          }

          // only search for filename in case of executable files
          if (extension === "exe") {
            const filename = lowerPath.split("\\").pop();
            // Use separator-aware check to avoid partial name matches (e.g. "mysteam.exe" != "steam.exe")
            if (filename && (e.path === filename || e.path.endsWith("\\" + filename))) {
              return true;
            }
          }
          return false;
        }) as Extract<IconPackEntry, { type: "unique" }>;
      }

      if (entry) {
        if (entry.redirect) {
          // break circular references
          if (__seen.has(entry.redirect)) {
            return null;
          }
          __seen.add(entry.redirect);
          return this.getIconEntry({ path: entry.redirect, __seen } as GetIconArgs);
        }

        if (entry.icon) {
          return entry;
        }
      }
    }

    // search by file extension
    if (!extension) {
      return null;
    }

    for (const pack of this._availableIconPacks.toReversed()) {
      const entry = pack.entries.find((e) => {
        return e.type === "shared" && e.extension === extension;
      });

      if (entry) {
        return entry;
      }
    }

    // If no icon is found in any icon pack, return null
    return null;
  }

  public getIconPath(args: GetIconArgs): IIcon | null {
    const entry = this.getIconEntry(args);
    return entry?.icon || null;
  }

  public getIcon({ path, umid }: GetIconArgs): IIcon | null {
    const iconPath = this.getIconPath({ path, umid });
    return iconPath ? resolveAsSrc(iconPath) : null;
  }

  /**
   * Will return the special missing icon path from the highest priority icon pack.
   * If no icon pack haves a missing icon, will return null.
   */
  public getMissingIconPath(): IIcon | null {
    for (const pack of this._availableIconPacks.toReversed()) {
      if (pack.missing) {
        return pack.missing;
      }
    }
    return null;
  }

  /**
   * Will return the special missing icon SRC from the highest priority icon pack.
   * If no icon pack haves a missing icon, will return null.
   */
  public getMissingIcon(): IIcon | null {
    const iconPath = this.getMissingIconPath();
    return iconPath ? resolveAsSrc(iconPath) : null;
  }

  /**
   * Will return the specific icon entry from the highest priority icon pack.
   * If no icon pack haves the searched icon, will return null.
   */
  public getCustomIconEntry(name: string): IconPackEntry | null {
    for (const pack of this._availableIconPacks.toReversed()) {
      const entry = pack.entries.find((e) => e.type === "custom" && e.key === name);
      if (entry) {
        return entry;
      }
    }
    return null;
  }

  /**
   * Will return the specific icon path from the highest priority icon pack.
   * If no icon pack haves the searched icon, will return null.
   */
  public getCustomIconPath(name: string): IIcon | null {
    const entry = this.getCustomIconEntry(name);
    return entry?.icon || null;
  }

  /**
   * Will return the specific icon SRC from the highest priority icon pack.
   * If no icon pack haves the searched icon, will return null.
   */
  public getCustomIcon(name: string): IIcon | null {
    const iconPath = this.getCustomIconPath(name);
    return iconPath ? resolveAsSrc(iconPath) : null;
  }

  /**
   * This method doesn't take in care icon packs, just extracts the inherited icon into system's icon pack
   * if it's not already there.
   *
   * @param filePath The path to the app could be umid o full path
   * @example
   * const iconPath = instance.extractIcon({
   *   path: "C:\\Program Files\\Steam\\steam.exe"
   * });
   * const iconPath = instance.extractIcon({
   *   umid: "DockBar.DockBarApp_p6yyn03m1894e!App"
   * });
   */
  public static requestIconExtraction(obj: GetIconArgs): Promise<void> {
    return invoke(DockBarCommand.GetIcon, obj);
  }

  /**
   * This will delete all stored icons on the system icon pack.\
   * All icons should be regenerated after calling this method.
   */
  public static clearCachedIcons(): Promise<void> {
    return invoke(DockBarCommand.StateDeleteCachedIcons);
  }
}

function resolveIcon(parent: string, icon: IIcon): IIcon {
  return {
    base: icon.base ? `${parent}\\${icon.base}` : null,
    light: icon.light ? `${parent}\\${icon.light}` : null,
    dark: icon.dark ? `${parent}\\${icon.dark}` : null,
    mask: icon.mask ? `${parent}\\${icon.mask}` : null,
    isAproximatelySquare: icon.isAproximatelySquare,
  };
}

function resolveAsSrc(icon: IIcon): IIcon {
  return {
    base: icon.base ? convertFileSrc(icon.base) : null,
    light: icon.light ? convertFileSrc(icon.light) : null,
    dark: icon.dark ? convertFileSrc(icon.dark) : null,
    mask: icon.mask ? convertFileSrc(icon.mask) : null,
    isAproximatelySquare: icon.isAproximatelySquare,
  };
}
