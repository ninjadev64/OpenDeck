import { writable, type Writable } from "svelte/store";

export const inspectedInstance: Writable<string | null> = writable(null);
