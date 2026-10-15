import type { DockBarCmdArgumentMap, DockBarCmdReturnMap, DockBarEventPayload } from "@dockbar/types";
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { type EventCallback, listen, type Options as ListenerOptions } from "@tauri-apps/api/event";

import { DockBarCommand } from "./commands.ts";
import { DockBarEvent } from "./events.ts";
import { getEventsChannel } from "./events_channel.ts";

type $keyof<Type> = [Type] extends [never] ? keyof Type : Type extends Type ? keyof Type : never;

type UnionToIntersection<Type> = {
  [Key in $keyof<Type>]: Extract<
    Type,
    {
      [key in Key]?: unknown;
    }
  >[Key];
};

type EmptyObject = Record<symbol, never>;

export type AllDockBarCommandArguments = UnionToIntersection<DockBarCmdArgumentMap>;
export type AllDockBarCommandReturns = UnionToIntersection<DockBarCmdReturnMap>;

/**
 * Will call to the background process
 * @args Command to be called
 * @args Command arguments
 * @return Result of the command
 */
export function invoke<T extends DockBarCommand>(
  ...args: EmptyObject extends Required<AllDockBarCommandArguments[T]> ? [command: T]
    : [command: T, args: AllDockBarCommandArguments[T]]
): Promise<AllDockBarCommandReturns[T]> {
  const [command, commandArgs] = args;
  return tauriInvoke(command, commandArgs);
}

export type UnSubscriber = () => void;
export type AllDockBarEventPayloads = UnionToIntersection<DockBarEventPayload>;

/**
 * Subscribes to a global event through the events websocket.
 * Resolves once the subscription is active, so events emitted after it are not missed.
 */
export function subscribe<T extends DockBarEvent>(
  event: T,
  cb: EventCallback<AllDockBarEventPayloads[T]>,
): Promise<UnSubscriber> {
  return getEventsChannel().subscribe(event, cb as EventCallback<unknown>);
}

/** Events in tauri causes memory leaks, so we use websockets instead */
export function __unsafe_tauri_subscribe<T extends DockBarEvent>(
  event: T,
  cb: EventCallback<AllDockBarEventPayloads[T]>,
  options?: ListenerOptions,
): Promise<UnSubscriber> {
  return listen(event, cb, options);
}

export { DockBarCommand, DockBarEvent };
