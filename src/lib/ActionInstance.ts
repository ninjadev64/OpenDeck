import type { Action } from "./Action";
import type { ActionState } from "./ActionState";

export type ActionInstance = {
	action: Action,
	context: string,
	states: ActionState[],
	current_state: number,
	settings: any,
	children: ActionInstance[] | null
};
