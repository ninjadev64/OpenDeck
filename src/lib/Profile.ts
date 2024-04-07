import type { ActionInstance } from "./ActionInstance";

export type Profile = {
	device: string,
	id: string,
	keys: ActionInstance[][],
	sliders: ActionInstance[][]
};
