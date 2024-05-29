import type { Context } from "./Context";

import { writable, type Writable } from "svelte/store";

export const inspectedInstance: Writable<string | null> = writable(null);

import { invoke } from "@tauri-apps/api";
let old: string | null = null;
inspectedInstance.subscribe(async (value) => {
	await invoke("switch_property_inspector", { old, new: value });
	old = value;
});

export const inspectedMultiAction: Writable<Context | null> = writable(null);

export const openContextMenu: Writable<{ context: Context, x: number, y: number } | null> = writable(null);
document.addEventListener("click", () => openContextMenu.set(null));
